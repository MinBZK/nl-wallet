import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../environment.dart';
import '../../../l10n/generated/app_localizations.dart';
import '../../data/service/announcement_service.dart';
import '../../domain/model/bloc/error_state.dart';
import '../../util/cast_util.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../wallet_constants.dart';
import '../common/widget/button/button_content.dart';
import '../common/widget/button/list_button.dart';
import '../common/widget/pin_header.dart';
import '../common/widget/wallet_scrollbar.dart';
import '../error/error_screen.dart';
import '../forgot_pin/forgot_pin_screen.dart';
import '../pin_blocked/pin_blocked_screen.dart';
import '../pin_timeout/pin_timeout_screen.dart';
import 'bloc/pin_bloc.dart';
import 'widget/pin_field.dart';
import 'widget/pin_keyboard.dart';

/// If the user has less then [kNonFinalRoundMLeftoverAttemptsMentionThreshold] attempts left
/// to enter the correct pin, we switch to showing the counter inside the warning dialog.
const kNonFinalRoundMLeftoverAttemptsMentionThreshold = 3;

/// Signature for a function that creates a widget while providing the leftover pin attempts.
/// [attempts] being null indicates that this is the first attempt.
/// [isFinalAttempt] being true indicates it's the final attempt (followed by the user being blocked, i.e. no more timeout)
typedef PinHeaderBuilder =
    Widget Function(
      BuildContext context,
      int? attemptsLeftInRound,
      //ignore: avoid_positional_boolean_parameters
      bool isFinalRound,
    );

/// Signature for a function that is called on any state change exposed by the [PinBloc]. When this method
/// is provided AND returns true for the given [PinState], the state is considered consumed and will not be handled
/// by the [PinPage] to trigger potential (navigation) events.
typedef PinStateInterceptor = bool Function(BuildContext context, PinState state);

/// Signature for a function that is called when the [PinBloc] exposes an [ErrorState]
typedef OnPinErrorCallback = void Function(BuildContext context, ErrorState state);

/// Signature for a function that is called when the user has entered the correct pin.
/// Provides an optional result T, which is the result of the CheckPinUseCase call.
typedef OnPinValidatedCallback<T> = void Function(T);

/// Provides pin validation and renders any errors based on the state from the nearest [PinBloc].
class PinPage extends StatelessWidget {
  /// Called when pin entry was successful.
  final OnPinValidatedCallback onPinValidated;

  /// When provided, replaces the default behaviour of the 'Forgot PIN?' button (in-page & dialog)
  final VoidCallback? onForgotPinPressed;

  /// Called when the user presses the biometrics key, setting this callback will make
  /// the 'biometrics' key appear on the [PinKeyboard].
  final VoidCallback? onBiometricUnlockRequested;

  /// Called for every state change exposed by the [PinBloc]. When [onStateChanged] is
  /// provided and it returns true, the event is not processed by this [PinPage].
  final PinStateInterceptor? onStateChanged;

  /// Called when the [PinBloc] exposes an [ErrorState] (i.e [PinValidateNetworkError]
  /// or [PinValidateGenericError]). When [onPinError] is provided these [ErrorState]s
  /// are no longer handled by the [PinPage].
  final OnPinErrorCallback? onPinError;

  /// Build a custom header, when null it defaults to [_defaultHeaderBuilder].
  final PinHeaderBuilder? headerBuilder;

  /// Draw a divider at the top of the screen when in landscape mode
  final bool showTopDivider;

  const PinPage({
    required this.onPinValidated,
    this.onStateChanged,
    this.onPinError,
    this.onForgotPinPressed,
    this.onBiometricUnlockRequested,
    this.headerBuilder,
    this.showTopDivider = false,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return BlocListener<PinBloc, PinState>(
      listener: _listenerForState,
      child: _buildBody(),
    );
  }

  Future<void> _listenerForState(BuildContext context, PinState state) async {
    _runEnteredDigitsAnnouncement(context, state);

    /// Check for state interceptions
    if (onStateChanged?.call(context, state) ?? false) return;
    if (onPinError != null && state is ErrorState) {
      onPinError!(context, state as ErrorState);
      return;
    }

    /// Process the state change
    switch (state) {
      case PinValidateSuccess():
        onPinValidated.call(state.result);
      case PinValidateTimeout():
        PinTimeoutScreen.show(context, state.expiryTime);
      case PinValidateBlocked():
        PinBlockedScreen.show(context);
      case PinValidateNetworkError():
        ErrorScreen.showNetwork(context, secured: false, networkError: tryCast(state));
      case PinValidateGenericError():
        ErrorScreen.showGeneric(context, secured: false);
      case PinValidateFailure():
        await _showErrorDialog(context, state);

      /// No need to handle these explicitly as events for now.
      case PinEntryInProgress():
      case PinValidateInProgress():
        break;
    }
  }

  void _runEnteredDigitsAnnouncement(BuildContext context, PinState state) {
    final AppLocalizations l10n = context.l10n;
    final AnnouncementService announcementService = context.read();
    switch (state) {
      case PinEntryInProgress():
        unawaited(
          Future.delayed(Environment.isTest ? Duration.zero : kDefaultAnnouncementDelay).then(
            (value) {
              if (state.afterBackspacePressed) {
                announcementService.announceEnteredDigits(l10n, state.enteredDigits);
              } else if (state.enteredDigits > 0 && state.enteredDigits < kPinDigits) {
                announcementService.announceEnteredDigits(l10n, state.enteredDigits);
              }
            },
          ),
        );
      case PinValidateInProgress():
        announcementService.announce(l10n.pinEnteredWCAGAnnouncement, assertiveness: Assertiveness.assertive);
      default:
        return;
    }
  }

  Widget _buildBody() {
    return OrientationBuilder(
      builder: (context, orientation) {
        switch (orientation) {
          case Orientation.portrait:
            return _buildPortrait(context);
          case Orientation.landscape:
            return _buildLandscape(context);
        }
      },
    );
  }

  Widget _buildPortrait(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        Expanded(child: _buildHeader(headerBuilder ?? _defaultHeaderBuilder)),
        _buildPinField(),
        SizedBox(height: context.reduceSpacing ? 6 : 18),
        _buildPinKeyboard(),
        SafeArea(
          child: _buildForgotCodeButton(context),
        ),
      ],
    );
  }

  Widget _buildLandscape(BuildContext context) {
    final leftSection = Column(
      children: [
        Expanded(
          child: SafeArea(
            right: false,
            top: false,
            bottom: false,
            child: _buildHeader(headerBuilder ?? _defaultHeaderBuilder),
          ),
        ),
        SafeArea(
          top: false,
          right: false,
          child: _buildForgotCodeButton(context),
        ),
      ],
    );
    final rightSection = Padding(
      padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
      child: SafeArea(
        left: false,
        right: true,
        top: false,
        bottom: false,
        child: Column(
          children: [
            _buildPinField(),
            const SizedBox(height: 16),
            Expanded(
              child: _buildPinKeyboard(),
            ),
          ],
        ),
      ),
    );
    return Column(
      mainAxisSize: MainAxisSize.max,
      children: [
        showTopDivider ? const Divider() : const SizedBox.shrink(),
        Expanded(
          child: Row(
            children: [
              Flexible(flex: 5, child: leftSection),
              const VerticalDivider(width: 1),
              Flexible(flex: 4, child: rightSection),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildHeader(PinHeaderBuilder builder) {
    // FIXME: Remove [MergeSemantics] once flutter issue is fixed.
    // Merging semantics fixes the (iOS) issue where only the scrollview can gain focus (without any announcement)
    // Can be removed if the related framework bug is fixed. https://github.com/flutter/flutter/issues/164483
    return MergeSemantics(
      child: WalletScrollbar(
        child: CustomScrollView(
          slivers: [
            SliverFillRemaining(
              hasScrollBody: false,
              child: BlocBuilder<PinBloc, PinState>(
                builder: (context, state) {
                  if (state is PinValidateFailure) {
                    return builder(context, state.attemptsLeftInRound, state.isFinalRound);
                  } else {
                    return builder(context, null, false);
                  }
                },
              ),
            ),
          ],
        ),
      ),
    );
  }

  /// Builds the default pin header, as shown on the 'unlock the app' screen.
  Widget _defaultHeaderBuilder(BuildContext context, int? attemptsLeftInRound, bool isFinalRound) {
    return PinHeader(
      title: context.l10n.pinScreenHeader,
      contentAlignment: context.isLandscape ? Alignment.centerLeft : Alignment.topCenter,
      textAlign: context.isLandscape ? TextAlign.start : TextAlign.center,
    );
  }

  Widget _buildPinField() {
    return BlocBuilder<PinBloc, PinState>(
      builder: (context, state) {
        return PinField(
          digits: kPinDigits,
          enteredDigits: _resolveEnteredDigits(state),
          state: _resolvePinFieldState(state),
        );
      },
    );
  }

  Widget _buildForgotCodeButton(BuildContext context) {
    return ListButton(
      mainAxisAlignment: context.isLandscape ? MainAxisAlignment.start : MainAxisAlignment.center,
      icon: const Icon(Icons.help_outline_rounded),
      onPressed: onForgotPinPressed ?? () => ForgotPinScreen.show(context),
      iconPosition: IconPosition.start,
      text: Text.rich(context.l10n.pinScreenForgotPinCta.toTextSpan(context)),
      dividerSide: DividerSide.top,
    );
  }

  Widget _buildPinKeyboard() {
    return BlocBuilder<PinBloc, PinState>(
      builder: (context, state) {
        return AnimatedOpacity(
          duration: kDefaultAnimationDuration,
          opacity: state is PinValidateInProgress ? 0.3 : 1,
          child: PinKeyboard(
            onKeyPressed: _digitKeysEnabled(state) ? (digit) => context.bloc.add(PinDigitPressed(digit)) : null,
            onBackspacePressed: _backspaceKeyEnabled(state)
                ? () => context.bloc.add(const PinBackspacePressed())
                : null,
            onBackspaceLongPressed: _backspaceKeyEnabled(state)
                ? () => context.bloc.add(const PinClearPressed())
                : null,
            onBiometricsPressed: onBiometricUnlockRequested,
          ),
        );
      },
    );
  }

  bool _digitKeysEnabled(PinState state) {
    return switch (state) {
      PinEntryInProgress() => true,
      PinValidateFailure() => true,
      PinValidateTimeout() => true,
      PinValidateNetworkError() => true,
      PinValidateGenericError() => true,
      PinValidateInProgress() => false,
      PinValidateSuccess() => false,
      PinValidateBlocked() => false,
    };
  }

  bool _backspaceKeyEnabled(PinState state) {
    return switch (state) {
      PinEntryInProgress() => true,
      PinValidateFailure() => true,
      PinValidateInProgress() => false,
      PinValidateSuccess() => false,
      PinValidateTimeout() => false,
      PinValidateBlocked() => false,
      PinValidateNetworkError() => false,
      PinValidateGenericError() => false,
    };
  }

  int _resolveEnteredDigits(PinState state) {
    return switch (state) {
      PinEntryInProgress() => state.enteredDigits,
      PinValidateInProgress() => kPinDigits,
      PinValidateSuccess() => kPinDigits,
      PinValidateFailure() => 0,
      PinValidateTimeout() => 0,
      PinValidateBlocked() => 0,
      PinValidateNetworkError() => 0,
      PinValidateGenericError() => 0,
    };
  }

  PinFieldState _resolvePinFieldState(PinState state) {
    if (state is PinValidateInProgress) return PinFieldState.loading;
    if (state is PinValidateFailure) return PinFieldState.error;
    return PinFieldState.idle;
  }

  Future<void> _showErrorDialog(BuildContext context, PinValidateFailure reason) async {
    final body = _pinErrorDialogBody(context, reason);
    return showPinErrorDialog(context, body, onForgotPinPressed: onForgotPinPressed);
  }

  static Future<void> showPinErrorDialog(
    BuildContext context,
    String description, {
    VoidCallback? onForgotPinPressed,
  }) async {
    final title = context.l10n.pinErrorDialogTitle;
    return showDialog<void>(
      context: context,
      barrierDismissible: false,
      builder: (BuildContext context) {
        return AlertDialog(
          scrollable: true,
          title: Text.rich(title.toTextSpan(context)),
          content: Text.rich(description.toTextSpan(context)),
          actions: <Widget>[
            TextButton(
              child: Text.rich(context.l10n.pinErrorDialogForgotCodeCta.toUpperCase().toTextSpan(context)),
              onPressed: () {
                Navigator.of(context).pop();
                if (onForgotPinPressed != null) {
                  onForgotPinPressed();
                } else {
                  ForgotPinScreen.show(context);
                }
              },
            ),
            TextButton(
              child: Text.rich(context.l10n.pinErrorDialogCloseCta.toUpperCase().toTextSpan(context)),
              onPressed: () => Navigator.of(context).pop(),
            ),
          ],
        );
      },
    );
  }

  String _pinErrorDialogBody(BuildContext context, PinValidateFailure reason) {
    if (reason.isFinalRound) {
      // Final round is a special case where the user has X attempts left before the app is blocked.
      if (reason.attemptsLeftInRound > 1) {
        return context.l10n.pinErrorDialogFinalRoundNonFinalAttempt(reason.attemptsLeftInRound);
      } else {
        return context.l10n.pinErrorDialogFinalRoundFinalAttempt;
      }
    } else {
      // Regular case where the user has X attempts left before the app is temporary blocked.
      switch (reason.attemptsLeftInRound) {
        case 1:
          return context.l10n.pinErrorDialogNonFinalRoundFinalAttempt;
        case < kNonFinalRoundMLeftoverAttemptsMentionThreshold:
          return context.l10n.pinErrorDialogNonFinalRoundNonFinalAttempt(reason.attemptsLeftInRound);
        default:
          return context.l10n.pinErrorDialogNonFinalRoundInitialAttempt;
      }
    }
  }
}

extension _PinPageExtensions on BuildContext {
  PinBloc get bloc => read<PinBloc>();
}
