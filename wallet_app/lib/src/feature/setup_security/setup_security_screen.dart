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
import '../common/widget/button/text_icon_button.dart';
import '../common/widget/fake_paging_animated_switcher.dart';
import '../error/error_screen.dart';
import 'bloc/setup_security_bloc.dart';
import 'page/setup_security_completed_page.dart';
import 'page/setup_security_pin_page.dart';

const _kSelectPinScreenKey = ValueKey('selectPinScreen');
const _kConfirmPinScreenKey = ValueKey('confirmPinScreen');

class SetupSecurityScreen extends StatelessWidget {
  const SetupSecurityScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      restorationId: 'setup_security_scaffold',
      appBar: AppBar(
        leading: _buildBackButton(context),
        title: Text(context.l10n.setupSecurityScreenTitle),
        actions: [
          _buildAboutAction(context),
        ],
      ),
      body: WillPopScope(
        onWillPop: () async {
          final bloc = context.read<SetupSecurityBloc>();
          if (bloc.state.canGoBack) bloc.add(SetupSecurityBackPressed());
          return !bloc.state.canGoBack;
        },
        child: Column(
          children: [
            _buildStepper(),
            Expanded(child: _buildPage()),
          ],
        ),
      ),
    );
  }

  Widget _buildStepper() {
    return BlocBuilder<SetupSecurityBloc, SetupSecurityState>(
      buildWhen: (prev, current) => prev.stepperProgress != current.stepperProgress,
      builder: (context, state) => AnimatedLinearProgressIndicator(progress: state.stepperProgress),
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
        _runAnnouncements(context, state);
      },
      builder: (context, state) {
        Widget result = switch (state) {
          SetupSecuritySelectPinInProgress() => _buildSelectPinPage(context, state),
          SetupSecuritySelectPinFailed() => _buildSelectPinErrorPage(context, state),
          SetupSecurityPinConfirmationInProgress() => _buildPinConfirmationPage(context, state),
          SetupSecurityPinConfirmationFailed() => _buildPinConfirmationErrorPage(context, state),
          SetupSecurityCreatingWallet() => _buildCreatingWallet(context, state),
          SetupSecurityCompleted() => _buildSetupCompletedPage(context, state),
          SetupSecurityGenericError() => _buildSetupFailed(context),
          SetupSecurityNetworkError() => _buildSetupFailed(context),
        };
        return SafeArea(child: FakePagingAnimatedSwitcher(animateBackwards: state.didGoBack, child: result));
      },
    );
  }

  void _runAnnouncements(BuildContext context, SetupSecurityState state) async {
    if (!MediaQuery.of(context).accessibleNavigation) return;
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

  Widget _buildSelectPinPage(BuildContext context, SetupSecuritySelectPinInProgress state) {
    return SetupSecurityPinPage(
      key: _kSelectPinScreenKey,
      content: Text(
        context.l10n.setupSecuritySelectPinPageTitle,
        style: context.textTheme.displaySmall,
        textAlign: TextAlign.center,
      ),
      enteredDigits: state.enteredDigits,
      onKeyPressed: (digit) => context.read<SetupSecurityBloc>().add(PinDigitPressed(digit)),
      onBackspacePressed: () => context.read<SetupSecurityBloc>().add(PinBackspacePressed()),
    );
  }

  Widget _buildSelectPinErrorPage(BuildContext context, SetupSecuritySelectPinFailed state) {
    String errorTitle = context.l10n.setupSecuritySelectPinErrorPageTitle;
    String errorDescription;
    switch (state.reason) {
      case PinValidationError.tooFewUniqueDigits:
        errorDescription = context.l10n.setupSecuritySelectPinErrorPageTooFewUniqueDigitsError;
        break;
      case PinValidationError.sequentialDigits:
        errorDescription = context.l10n.setupSecuritySelectPinErrorPageAscendingOrDescendingDigitsError;
        break;
      default:
        errorDescription = context.l10n.setupSecuritySelectPinErrorPageDefaultError;
    }
    return SetupSecurityPinPage(
      key: _kSelectPinScreenKey,
      content: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16),
        child: Column(
          children: [
            Text(
              errorTitle,
              style: context.textTheme.displaySmall?.copyWith(color: context.colorScheme.error),
              textAlign: TextAlign.center,
            ),
            Text(
              errorDescription,
              style: context.textTheme.bodyLarge?.copyWith(color: context.colorScheme.error),
              textAlign: TextAlign.center,
            ),
          ],
        ),
      ),
      enteredDigits: 0,
      onKeyPressed: (digit) => context.read<SetupSecurityBloc>().add(PinDigitPressed(digit)),
      onBackspacePressed: () => context.read<SetupSecurityBloc>().add(PinBackspacePressed()),
      isShowingError: true,
    );
  }

  Widget _buildPinConfirmationPage(BuildContext context, SetupSecurityPinConfirmationInProgress state) {
    return SetupSecurityPinPage(
      key: _kConfirmPinScreenKey,
      content: Text(
        context.l10n.setupSecurityConfirmationPageTitle,
        style: context.textTheme.displaySmall,
        textAlign: TextAlign.center,
      ),
      enteredDigits: state.enteredDigits,
      onKeyPressed: (digit) => context.read<SetupSecurityBloc>().add(PinDigitPressed(digit)),
      onBackspacePressed: () => context.read<SetupSecurityBloc>().add(PinBackspacePressed()),
    );
  }

  Widget _buildPinConfirmationErrorPage(BuildContext context, SetupSecurityPinConfirmationFailed state) {
    final titleStyle = context.textTheme.displaySmall?.copyWith(color: context.colorScheme.error);
    final descriptionStyle = context.textTheme.bodyLarge?.copyWith(color: context.colorScheme.error);
    Widget content;
    if (state.retryAllowed) {
      content = Column(
        children: [
          Text(
            context.l10n.setupSecurityConfirmationErrorPageTitle,
            style: titleStyle,
            textAlign: TextAlign.center,
          ),
          Text(
            context.l10n.setupSecurityConfirmationErrorPageDescription,
            style: descriptionStyle,
            textAlign: TextAlign.center,
          ),
        ],
      );
    } else {
      content = Column(
        children: [
          Text(
            context.l10n.setupSecurityConfirmationErrorPageFatalTitle,
            style: titleStyle,
            textAlign: TextAlign.center,
          ),
          Text(
            context.l10n.setupSecurityConfirmationErrorPageFatalDescription,
            style: descriptionStyle,
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 24),
          TextIconButton(
            key: const Key('setupSecurityConfirmationErrorPageFatalCta'),
            child: Text(context.l10n.setupSecurityConfirmationErrorPageFatalCta),
            onPressed: () => context.read<SetupSecurityBloc>().add(SetupSecurityBackPressed()),
          ),
        ],
      );
    }
    return SetupSecurityPinPage(
      key: _kConfirmPinScreenKey,
      showInput: state.retryAllowed,
      content: content,
      enteredDigits: 0,
      onKeyPressed: (digit) => context.read<SetupSecurityBloc>().add(PinDigitPressed(digit)),
      onBackspacePressed: () => context.read<SetupSecurityBloc>().add(PinBackspacePressed()),
      isShowingError: true,
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
}
