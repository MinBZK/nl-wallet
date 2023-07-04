import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../util/extension/build_context_extension.dart';
import '../../wallet_constants.dart';
import '../common/widget/button/text_icon_button.dart';
import '../common/widget/wallet_logo.dart';
import '../error/error_screen.dart';
import '../forgot_pin/forgot_pin_screen.dart';
import '../pin_blocked/pin_blocked_screen.dart';
import '../pin_timeout/pin_timeout_screen.dart';
import 'bloc/pin_bloc.dart';
import 'widget/pin_field.dart';
import 'widget/pin_keyboard.dart';

/// Signature for a function that creates a widget while providing the leftover pin attempts.
/// [attempts] being null indicates that this is the first attempt.
/// [isFinalAttempt] being true indicates it's the final attempt (followed by the user being blocked, i.e. no more timeout)
typedef PinHeaderBuilder = Widget Function(BuildContext context, int? attempts, bool isFinalAttempt);

/// The required minimum height in the header to be able to show the logo
const _kHeaderHeightLogoCutOff = 180;

/// Provides pin validation and renders any errors based on the state from the nearest [PinBloc].
class PinPage extends StatelessWidget {
  final VoidCallback? onPinValidated;
  final PinHeaderBuilder? headerBuilder;

  const PinPage({
    this.onPinValidated,
    this.headerBuilder,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocListener<PinBloc, PinState>(
      listener: (context, state) {
        if (state is PinEntryInProgress) {
          if (state.afterBackspacePressed) {
            announceEnteredDigits(context, state.enteredDigits);
          } else if (state.enteredDigits > 0 && state.enteredDigits < kPinDigits) {
            announceEnteredDigits(context, state.enteredDigits);
          }
        }
        if (state is PinValidateSuccess) {
          SemanticsService.announce(context.l10n.pinScreenWCAGPinOkWalletUnlockedAnnouncement, TextDirection.ltr);
          onPinValidated?.call();
        }
        if (state is PinValidateServerError) {
          ErrorScreen.showGeneric(context, secured: false);
        }
        if (state is PinValidateTimeout) {
          PinTimeoutScreen.show(context, state.expiryTime);
        }
        if (state is PinValidateBlocked) {
          PinBlockedScreen.show(context);
        }
      },
      child: OrientationBuilder(
        builder: (context, orientation) {
          switch (orientation) {
            case Orientation.portrait:
              return _buildPortrait();
            case Orientation.landscape:
              return _buildLandscape();
          }
        },
      ),
    );
  }

  Widget _buildPortrait() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        Expanded(child: _buildHeader(headerBuilder ?? _defaultHeaderBuilder)),
        _buildPinField(),
        const SizedBox(height: 18),
        _buildForgotCodeButton(),
        const SizedBox(height: 18),
        _buildPinKeyboard(),
      ],
    );
  }

  Widget _buildLandscape() {
    return Row(
      children: [
        Expanded(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              _buildHeader(headerBuilder ?? _defaultHeaderBuilder),
              const SizedBox(height: 24),
              _buildPinField(),
              const SizedBox(height: 18),
              _buildForgotCodeButton(),
            ],
          ),
        ),
        Expanded(
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 16),
            child: _buildPinKeyboard(),
          ),
        ),
      ],
    );
  }

  Widget _buildHeader(PinHeaderBuilder builder) {
    return BlocBuilder<PinBloc, PinState>(
      builder: (context, state) {
        if (state is PinValidateFailure) {
          return builder(context, state.leftoverAttempts, state.isFinalAttempt);
        } else {
          return builder(context, null, false);
        }
      },
    );
  }

  Widget _defaultHeaderBuilder(BuildContext context, int? attempts, bool isFinalAttempt) {
    if (context.isLandscape) return _buildTextHeader(context, attempts, isFinalAttempt);
    return LayoutBuilder(builder: (context, constraints) {
      if (constraints.maxHeight < _kHeaderHeightLogoCutOff) return _buildTextHeader(context, attempts, isFinalAttempt);
      return Column(
        children: [
          const Spacer(),
          const WalletLogo(size: 80),
          const SizedBox(height: 24),
          _buildTextHeader(context, attempts, isFinalAttempt),
          const Spacer(),
        ],
      );
    });
  }

  Widget _buildTextHeader(BuildContext context, int? attempts, bool isFinalAttempt) {
    var headerText = context.l10n.pinScreenHeader;
    var descriptionText = '';
    bool useErrorColor = attempts != null || isFinalAttempt;

    if (attempts != null) {
      headerText = context.l10n.pinScreenErrorHeader;
      descriptionText = context.l10n.pinScreenAttemptsCount(attempts);
    }
    if (isFinalAttempt) {
      headerText = context.l10n.pinScreenErrorHeader;
      descriptionText = context.l10n.pinScreenFinalAttempt;
    }

    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 24),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Text(
            headerText,
            style: context.textTheme.displaySmall?.copyWith(color: useErrorColor ? context.colorScheme.error : null),
            textAlign: TextAlign.center,
          ),
          Text(
            descriptionText,
            style: context.textTheme.bodyLarge?.copyWith(color: useErrorColor ? context.colorScheme.error : null),
            textAlign: TextAlign.center,
          ),
        ],
      ),
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

  Widget _buildForgotCodeButton() {
    return BlocBuilder<PinBloc, PinState>(
      builder: (context, state) {
        final buttonEnabled = state is PinEntryInProgress || state is PinValidateFailure;
        return TextIconButton(
          onPressed: buttonEnabled ? () => ForgotPinScreen.show(context) : null,
          child: Text(context.l10n.pinScreenForgotPinCta),
        );
      },
    );
  }

  Widget _buildPinKeyboard() {
    return BlocBuilder<PinBloc, PinState>(
      builder: (context, state) {
        return AnimatedOpacity(
          duration: kDefaultAnimationDuration,
          opacity: state is PinValidateInProgress ? 0.3 : 1,
          child: PinKeyboard(
            onKeyPressed:
                _digitKeysEnabled(state) ? (digit) => context.read<PinBloc>().add(PinDigitPressed(digit)) : null,
            onBackspacePressed:
                _backspaceKeyEnabled(state) ? () => context.read<PinBloc>().add(const PinBackspacePressed()) : null,
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
      PinValidateServerError() => true,
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
      PinValidateServerError() => false,
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
      PinValidateServerError() => 0,
    };
  }

  PinFieldState _resolvePinFieldState(PinState state) {
    if (state is PinValidateInProgress) return PinFieldState.loading;
    if (state is PinValidateFailure) return PinFieldState.error;
    return PinFieldState.idle;
  }

  void announceEnteredDigits(BuildContext context, int enteredDigits) {
    SemanticsService.announce(
      context.l10n.setupSecurityScreenWCAGEnteredDigitsAnnouncement(enteredDigits, kPinDigits),
      TextDirection.ltr,
    );
  }
}
