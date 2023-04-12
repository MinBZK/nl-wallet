import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../domain/model/pin/pin_validation_error.dart';
import '../../wallet_constants.dart';
import '../../wallet_routes.dart';
import '../common/widget/animated_linear_progress_indicator.dart';
import '../common/widget/animated_visibility_back_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/fake_paging_animated_switcher.dart';
import '../common/widget/text_icon_button.dart';
import 'bloc/setup_security_bloc.dart';
import 'page/setup_security_completed_page.dart';
import 'page/setup_security_pin_page.dart';

const _kSelectPinKey = ValueKey('select_pin');
const _kConfirmPinKey = ValueKey('confirm_pin');

class SetupSecurityScreen extends StatelessWidget {
  const SetupSecurityScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      restorationId: 'setup_security_scaffold',
      appBar: AppBar(
        title: Text(AppLocalizations.of(context).setupSecurityScreenTitle),
        leading: _buildBackButton(context),
      ),
      body: WillPopScope(
        onWillPop: () async {
          final bloc = context.read<SetupSecurityBloc>();
          if (bloc.state.canGoBack) bloc.add(SetupSecurityBackPressed());
          return false;
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
    return ExcludeSemantics(
      child: BlocBuilder<SetupSecurityBloc, SetupSecurityState>(
        buildWhen: (prev, current) => prev.stepperProgress != current.stepperProgress,
        builder: (context, state) => AnimatedLinearProgressIndicator(progress: state.stepperProgress),
      ),
    );
  }

  Widget _buildPage() {
    return BlocConsumer<SetupSecurityBloc, SetupSecurityState>(
      listener: (context, state) async {
        if (!MediaQuery.of(context).accessibleNavigation) return;
        final locale = AppLocalizations.of(context);
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
      },
      builder: (context, state) {
        Widget? result;
        if (state is SetupSecuritySelectPinInProgress) result = _buildSelectPinPage(context, state);
        if (state is SetupSecuritySelectPinFailed) result = _buildSelectPinErrorPage(context, state);
        if (state is SetupSecurityPinConfirmationInProgress) result = _buildPinConfirmationPage(context, state);
        if (state is SetupSecurityPinConfirmationFailed) result = _buildPinConfirmationErrorPage(context, state);
        if (state is SetupSecurityCreatingWallet) result = _buildCreatingWallet(context, state);
        if (state is SetupSecurityCompleted) result = _buildSetupCompletedPage(context, state);
        if (state is SetupSecurityFailure) result = _buildSetupFailed(context);
        if (result == null) throw UnsupportedError('Unknown state: $state');
        return FakePagingAnimatedSwitcher(animateBackwards: state.didGoBack, child: result);
      },
    );
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

  Widget _buildSelectPinPage(BuildContext context, SetupSecuritySelectPinInProgress state) {
    return SetupSecurityPinPage(
      key: _kSelectPinKey,
      content: Text(
        AppLocalizations.of(context).setupSecuritySelectPinPageTitle,
        style: Theme.of(context).textTheme.displaySmall,
        textAlign: TextAlign.center,
      ),
      enteredDigits: state.enteredDigits,
      onKeyPressed: (digit) => context.read<SetupSecurityBloc>().add(PinDigitPressed(digit)),
      onBackspacePressed: () => context.read<SetupSecurityBloc>().add(PinBackspacePressed()),
    );
  }

  Widget _buildSelectPinErrorPage(BuildContext context, SetupSecuritySelectPinFailed state) {
    final locale = AppLocalizations.of(context);
    String errorTitle = locale.setupSecuritySelectPinErrorPageTitle;
    String errorDescription;
    switch (state.reason) {
      case PinValidationError.tooFewUniqueDigits:
        errorDescription = locale.setupSecuritySelectPinErrorPageTooFewUniqueDigitsError;
        break;
      case PinValidationError.sequentialDigits:
        errorDescription = locale.setupSecuritySelectPinErrorPageAscendingOrDescendingDigitsError;
        break;
      default:
        errorDescription = locale.setupSecuritySelectPinErrorPageDefaultError;
    }
    return SetupSecurityPinPage(
      key: _kSelectPinKey,
      content: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16),
        child: Column(
          children: [
            Text(
              errorTitle,
              style: Theme.of(context).textTheme.displaySmall?.copyWith(color: Theme.of(context).colorScheme.error),
              textAlign: TextAlign.center,
            ),
            Text(
              errorDescription,
              style: Theme.of(context).textTheme.bodyLarge?.copyWith(color: Theme.of(context).colorScheme.error),
              textAlign: TextAlign.center,
            ),
          ],
        ),
      ),
      enteredDigits: 0,
      onKeyPressed: (digit) => context.read<SetupSecurityBloc>().add(PinDigitPressed(digit)),
      onBackspacePressed: () => context.read<SetupSecurityBloc>().add(PinBackspacePressed()),
    );
  }

  Widget _buildPinConfirmationPage(BuildContext context, SetupSecurityPinConfirmationInProgress state) {
    return SetupSecurityPinPage(
      key: _kConfirmPinKey,
      content: Text(
        AppLocalizations.of(context).setupSecurityConfirmationPageTitle,
        style: Theme.of(context).textTheme.displaySmall,
        textAlign: TextAlign.center,
      ),
      enteredDigits: state.enteredDigits,
      onKeyPressed: (digit) => context.read<SetupSecurityBloc>().add(PinDigitPressed(digit)),
      onBackspacePressed: () => context.read<SetupSecurityBloc>().add(PinBackspacePressed()),
    );
  }

  Widget _buildPinConfirmationErrorPage(BuildContext context, SetupSecurityPinConfirmationFailed state) {
    final locale = AppLocalizations.of(context);
    final titleStyle = Theme.of(context).textTheme.displaySmall?.copyWith(color: Theme.of(context).colorScheme.error);
    final descriptionStyle =
        Theme.of(context).textTheme.bodyLarge?.copyWith(color: Theme.of(context).colorScheme.error);
    Widget content;
    if (state.retryAllowed) {
      content = Column(
        children: [
          Text(
            locale.setupSecurityConfirmationErrorPageTitle,
            style: titleStyle,
            textAlign: TextAlign.center,
          ),
          Text(
            locale.setupSecurityConfirmationErrorPageDescription,
            style: descriptionStyle,
            textAlign: TextAlign.center,
          ),
        ],
      );
    } else {
      content = Column(
        children: [
          Text(
            locale.setupSecurityConfirmationErrorPageFatalTitle,
            style: titleStyle,
            textAlign: TextAlign.center,
          ),
          Text(
            locale.setupSecurityConfirmationErrorPageFatalDescription,
            style: descriptionStyle,
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 24),
          TextIconButton(
            child: Text(locale.setupSecurityConfirmationErrorPageFatalCta),
            onPressed: () => context.read<SetupSecurityBloc>().add(SetupSecurityBackPressed()),
          ),
        ],
      );
    }
    return SetupSecurityPinPage(
      key: _kConfirmPinKey,
      showInput: state.retryAllowed,
      content: content,
      enteredDigits: 0,
      onKeyPressed: (digit) => context.read<SetupSecurityBloc>().add(PinDigitPressed(digit)),
      onBackspacePressed: () => context.read<SetupSecurityBloc>().add(PinBackspacePressed()),
    );
  }

  Widget _buildCreatingWallet(BuildContext context, SetupSecurityCreatingWallet state) =>
      const CenteredLoadingIndicator();

  Widget _buildSetupCompletedPage(BuildContext context, SetupSecurityCompleted state) {
    return SetupSecurityCompletedPage(
      onSetupWalletPressed: () =>
          Navigator.restorablePushReplacementNamed(context, WalletRoutes.walletPersonalizeRoute),
    );
  }

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
              child: Text(AppLocalizations.of(context).generalRetry),
            ),
          )
        ],
      ),
    );
  }

  void announceEnteredDigits(BuildContext context, int enteredDigits) {
    var locale = AppLocalizations.of(context);
    SemanticsService.announce(
      locale.setupSecurityScreenWCAGEnteredDigitsAnnouncement(enteredDigits, kPinDigits),
      TextDirection.ltr,
    );
  }
}
