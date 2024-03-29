import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/model/pin/pin_validation_error.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/cast_util.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_constants.dart';
import '../common/page/generic_loading_page.dart';
import '../common/widget/animated_linear_progress_indicator.dart';
import '../common/widget/button/animated_visibility_back_button.dart';
import '../common/widget/fake_paging_animated_switcher.dart';
import '../common/widget/wallet_app_bar.dart';
import '../error/error_screen.dart';
import 'bloc/setup_security_bloc.dart';
import 'page/setup_security_completed_page.dart';
import 'page/setup_security_pin_page.dart';

const _kSelectPinScreenKey = ValueKey('selectPinScreen');
const _kConfirmPinScreenKey = ValueKey('confirmPinScreen');

class SetupSecurityScreen extends StatelessWidget {
  const SetupSecurityScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      restorationId: 'setup_security_scaffold',
      appBar: WalletAppBar(
        leading: _buildBackButton(context),
        actions: [_buildAboutAction(context)],
      ),
      body: PopScope(
        canPop: !context.bloc.state.canGoBack,
        onPopInvoked: (didPop) {
          if (!didPop) context.bloc.add(SetupSecurityBackPressed());
        },
        child: SafeArea(
          child: Column(
            children: [
              _buildStepper(),
              Expanded(child: _buildPage()),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildStepper() {
    return BlocBuilder<SetupSecurityBloc, SetupSecurityState>(
      buildWhen: (prev, current) => prev.stepperProgress != current.stepperProgress,
      builder: (context, state) => Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16),
        child: AnimatedLinearProgressIndicator(progress: state.stepperProgress),
      ),
    );
  }

  Widget _buildPage() {
    return BlocConsumer<SetupSecurityBloc, SetupSecurityState>(
      listener: (context, state) async {
        final String errorScreenTitle = context.l10n.setupSecurityScreenTitle;
        if (state is SetupSecurityGenericError) {
          ErrorScreen.showGeneric(context, title: errorScreenTitle, secured: false);
        }
        if (state is SetupSecurityNetworkError) {
          ErrorScreen.showNetwork(context, title: errorScreenTitle, networkError: tryCast(state), secured: false);
        }
        if (state is SetupSecuritySelectPinFailed) {
          _showErrorDialog(context, state.reason).then((_) => context.bloc.add(PinBackspacePressed()));
        }
        if (state is SetupSecurityPinConfirmationFailed) {
          _showConfirmationErrorDialog(context, state.retryAllowed).then((_) {
            context.bloc.add(state.retryAllowed ? PinBackspacePressed() : SetupSecurityRetryPressed());
          });
        }
        _runAnnouncements(context, state);
      },
      builder: (context, state) {
        Widget result = switch (state) {
          SetupSecuritySelectPinInProgress() => _buildSelectPinPage(context, enteredDigits: state.enteredDigits),
          SetupSecuritySelectPinFailed() => _buildSelectPinPage(context, enteredDigits: kPinDigits),
          SetupSecurityPinConfirmationInProgress() =>
            _buildPinConfirmationPage(context, enteredDigits: state.enteredDigits),
          SetupSecurityPinConfirmationFailed() => _buildPinConfirmationPage(context, enteredDigits: kPinDigits),
          SetupSecurityCreatingWallet() => _buildCreatingWallet(context, state),
          SetupSecurityCompleted() => _buildSetupCompletedPage(context, state),
          SetupSecurityGenericError() => _buildSetupFailed(context),
          SetupSecurityNetworkError() => _buildSetupFailed(context),
        };
        return FakePagingAnimatedSwitcher(animateBackwards: state.didGoBack, child: result);
      },
    );
  }

  void _runAnnouncements(BuildContext context, SetupSecurityState state) async {
    if (!context.mediaQuery.accessibleNavigation) return;
    final locale = context.l10n;
    if (state is SetupSecuritySelectPinInProgress) {
      if (state.afterBackspacePressed) {
        announceEnteredDigits(context, state.enteredDigits);
      } else if (state.enteredDigits > 0 && state.enteredDigits < kPinDigits) {
        announceEnteredDigits(context, state.enteredDigits);
      }
    }
    if (state is SetupSecurityPinConfirmationInProgress) {
      if (state.afterBackspacePressed) {
        announceEnteredDigits(context, state.enteredDigits);
      } else if (state.enteredDigits == 0) {
        await Future.delayed(const Duration(seconds: 1));
        SemanticsService.announce(locale.setupSecurityScreenWCAGPinChosenAnnouncement, TextDirection.ltr);
      } else if (state.enteredDigits > 0 && state.enteredDigits < kPinDigits) {
        announceEnteredDigits(context, state.enteredDigits);
      }
    }
    if (state is SetupSecuritySelectPinFailed) {
      SemanticsService.announce(locale.setupSecurityScreenWCAGPinTooSimpleAnnouncement, TextDirection.ltr);
    }
    if (state is SetupSecurityPinConfirmationFailed) {
      SemanticsService.announce(locale.setupSecurityScreenWCAGPinConfirmationFailedAnnouncement, TextDirection.ltr);
    }
  }

  Widget _buildBackButton(BuildContext context) {
    return BlocBuilder<SetupSecurityBloc, SetupSecurityState>(
      builder: (context, state) {
        return AnimatedVisibilityBackButton(
          visible: state.canGoBack,
          onPressed: () => context.read<SetupSecurityBloc>().add(SetupSecurityBackPressed()),
        );
      },
    );
  }

  Widget _buildAboutAction(BuildContext context) {
    return BlocBuilder<SetupSecurityBloc, SetupSecurityState>(
      builder: (context, state) {
        if (state is SetupSecurityCompleted) return const SizedBox.shrink();
        return IconButton(
          onPressed: () => Navigator.of(context).restorablePushNamed(WalletRoutes.aboutRoute),
          icon: const Icon(Icons.info_outline),
          tooltip: context.l10n.setupSecurityScreenAboutAppTooltip,
        );
      },
    );
  }

  Widget _buildSelectPinPage(BuildContext context, {required int enteredDigits}) {
    return SetupSecurityPinPage(
      key: _kSelectPinScreenKey,
      title: context.l10n.setupSecuritySelectPinPageTitle,
      enteredDigits: enteredDigits,
      onKeyPressed: (digit) => context.read<SetupSecurityBloc>().add(PinDigitPressed(digit)),
      onBackspacePressed: () => context.read<SetupSecurityBloc>().add(PinBackspacePressed()),
    );
  }

  Widget _buildPinConfirmationPage(BuildContext context, {required int enteredDigits}) {
    return SetupSecurityPinPage(
      key: _kConfirmPinScreenKey,
      title: context.l10n.setupSecurityConfirmationPageTitle,
      enteredDigits: enteredDigits,
      onKeyPressed: (digit) => context.read<SetupSecurityBloc>().add(PinDigitPressed(digit)),
      onBackspacePressed: () => context.read<SetupSecurityBloc>().add(PinBackspacePressed()),
    );
  }

  Widget _buildCreatingWallet(BuildContext context, SetupSecurityCreatingWallet state) {
    return GenericLoadingPage(
      title: context.l10n.setupSecurityLoadingPageTitle,
      description: context.l10n.setupSecurityLoadingPageDescription,
    );
  }

  Widget _buildSetupCompletedPage(BuildContext context, SetupSecurityCompleted state) {
    return SetupSecurityCompletedPage(
      key: const Key('setupSecurityCompletedPage'),
      onSetupWalletPressed: () =>
          Navigator.restorablePushReplacementNamed(context, WalletRoutes.walletPersonalizeRoute),
    );
  }

  /// This is more a placeholder/fallback over anything else.
  /// Whenever the user is hit with a [SetupSecurityGenericError] or [SetupSecurityNetworkError]
  /// this is built, but the listener should trigger the [ErrorScreen] while the bloc resets
  /// the flow so the user can try again. That said, to be complete we need to build something
  /// in this state, hence this method is kept around.
  Widget _buildSetupFailed(BuildContext context) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.error_outline),
          const SizedBox(height: 16),
          IntrinsicWidth(
            child: ElevatedButton(
              onPressed: () => context.read<SetupSecurityBloc>().add(SetupSecurityRetryPressed()),
              child: Text(context.l10n.generalRetry),
            ),
          )
        ],
      ),
    );
  }

  void announceEnteredDigits(BuildContext context, int enteredDigits) {
    SemanticsService.announce(
      context.l10n.setupSecurityScreenWCAGEnteredDigitsAnnouncement(enteredDigits, kPinDigits),
      TextDirection.ltr,
    );
  }

  Future<void> _showErrorDialog(BuildContext context, PinValidationError reason) async {
    final title = switch (reason) {
      PinValidationError.tooFewUniqueDigits => context.l10n.setupSecuritySelectPinErrorPageTitle,
      PinValidationError.sequentialDigits => context.l10n.setupSecuritySelectPinErrorPageTitle,
      PinValidationError.other => context.l10n.setupSecuritySelectPinErrorPageTitle,
    };
    final body = switch (reason) {
      PinValidationError.tooFewUniqueDigits => context.l10n.setupSecuritySelectPinErrorPageTooFewUniqueDigitsError,
      PinValidationError.sequentialDigits =>
        context.l10n.setupSecuritySelectPinErrorPageAscendingOrDescendingDigitsError,
      PinValidationError.other => context.l10n.setupSecuritySelectPinErrorPageDefaultError,
    };
    return showDialog<void>(
      context: context,
      barrierDismissible: true,
      builder: (BuildContext context) {
        return AlertDialog(
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(16),
          ),
          title: Text(title, style: context.textTheme.displayMedium),
          content: SingleChildScrollView(
            child: Text(body, style: context.textTheme.bodyLarge),
          ),
          actions: <Widget>[
            TextButton(
              child: Text(context.l10n.generalOkCta),
              onPressed: () => Navigator.of(context).pop(),
            ),
          ],
        );
      },
    );
  }

  Future<void> _showConfirmationErrorDialog(BuildContext context, bool retryAllowed) async {
    return showDialog<void>(
      context: context,
      barrierDismissible: retryAllowed,
      builder: (BuildContext context) {
        return AlertDialog(
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(16),
          ),
          title: Text(
            retryAllowed
                ? context.l10n.setupSecurityConfirmationErrorPageTitle
                : context.l10n.setupSecurityConfirmationErrorPageFatalTitle,
            style: context.textTheme.displayMedium,
          ),
          content: SingleChildScrollView(
            child: Text(
              retryAllowed
                  ? context.l10n.setupSecurityConfirmationErrorPageDescription
                  : context.l10n.setupSecurityConfirmationErrorPageFatalDescription,
              style: context.textTheme.bodyLarge,
            ),
          ),
          actions: <Widget>[
            TextButton(
              child: Text(
                retryAllowed ? context.l10n.generalOkCta : context.l10n.setupSecurityConfirmationErrorPageFatalCta,
              ),
              onPressed: () => Navigator.of(context).pop(),
            ),
          ],
        );
      },
    );
  }
}

extension _SetupSecurityScreenExtension on BuildContext {
  SetupSecurityBloc get bloc => read<SetupSecurityBloc>();
}
