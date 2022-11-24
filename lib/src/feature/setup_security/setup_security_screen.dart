import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../wallet_routes.dart';
import '../common/widget/animated_linear_progress_indicator.dart';
import '../common/widget/animated_visibility_back_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/fake_paging_animated_switcher.dart';
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
      restorationId: 'setup_pin_scaffold',
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
    return BlocBuilder<SetupSecurityBloc, SetupSecurityState>(
      buildWhen: (prev, current) => prev.stepperProgress != current.stepperProgress,
      builder: (context, state) => AnimatedLinearProgressIndicator(progress: state.stepperProgress),
    );
  }

  Widget _buildPage() {
    return BlocBuilder<SetupSecurityBloc, SetupSecurityState>(
      builder: (context, state) {
        Widget? result;
        if (state is SetupSecuritySelectPinInProgress) result = _buildSelectPinPage(context, state);
        if (state is SetupSecuritySelectPinFailed) result = _buildSelectPinErrorPage(context, state);
        if (state is SetupSecurityPinConfirmationInProgress) result = _buildPinConfirmationPage(context, state);
        if (state is SetupSecurityPinConfirmationFailed) result = _buildPinConfirmationErrorPage(context, state);
        if (state is SetupSecurityCreatingWallet) result = _buildCreatingWallet(context, state);
        if (state is SetupSecurityCompleted) result = _buildSetupCompletedPage(context, state);
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
        style: Theme.of(context).textTheme.headline3,
      ),
      enteredDigits: state.enteredDigits,
      onKeyPressed: (digit) => context.read<SetupSecurityBloc>().add(PinDigitPressed(digit)),
      onBackspacePressed: () => context.read<SetupSecurityBloc>().add(PinBackspacePressed()),
    );
  }

  Widget _buildSelectPinErrorPage(BuildContext context, SetupSecuritySelectPinFailed state) {
    return SetupSecurityPinPage(
      key: _kSelectPinKey,
      content: Column(
        children: [
          Text(
            AppLocalizations.of(context).setupSecuritySelectPinErrorPageTitle,
            style: Theme.of(context).textTheme.headline3?.copyWith(color: Theme.of(context).errorColor),
          ),
          Text(
            AppLocalizations.of(context).setupSecuritySelectPinErrorPageDescription,
            style: Theme.of(context).textTheme.bodyText1?.copyWith(color: Theme.of(context).errorColor),
          ),
        ],
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
        style: Theme.of(context).textTheme.headline3,
      ),
      enteredDigits: state.enteredDigits,
      onKeyPressed: (digit) => context.read<SetupSecurityBloc>().add(PinDigitPressed(digit)),
      onBackspacePressed: () => context.read<SetupSecurityBloc>().add(PinBackspacePressed()),
    );
  }

  Widget _buildPinConfirmationErrorPage(BuildContext context, SetupSecurityPinConfirmationFailed state) {
    return SetupSecurityPinPage(
      key: _kConfirmPinKey,
      content: Column(
        children: [
          Text(
            AppLocalizations.of(context).setupSecurityConfirmationErrorPageTitle,
            style: Theme.of(context).textTheme.headline3?.copyWith(color: Theme.of(context).errorColor),
          ),
          Text(
            AppLocalizations.of(context).setupSecurityConfirmationErrorPageDescription,
            style: Theme.of(context).textTheme.bodyText1?.copyWith(color: Theme.of(context).errorColor),
          ),
        ],
      ),
      enteredDigits: 0,
      onKeyPressed: (digit) => context.read<SetupSecurityBloc>().add(PinDigitPressed(digit)),
      onBackspacePressed: () => context.read<SetupSecurityBloc>().add(PinBackspacePressed()),
    );
  }

  Widget _buildCreatingWallet(BuildContext context, SetupSecurityCreatingWallet state) =>
      const CenteredLoadingIndicator();

  Widget _buildSetupCompletedPage(BuildContext context, SetupSecurityCompleted state) {
    return SetupSecurityCompletedPage(
      onSetupWalletPressed: () => Navigator.pushReplacementNamed(context, WalletRoutes.homeRoute),
    );
  }
}
