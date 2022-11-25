import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../wallet_constants.dart';
import '../../../wallet_routes.dart';
import '../../common/widget/animated_linear_progress_indicator.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/fake_paging_animated_switcher.dart';
import '../../common/widget/placeholder_screen.dart';
import '../../wallet/personalize/bloc/wallet_personalize_bloc.dart';
import 'page/wallet_personalize_check_data_offering_page.dart';
import 'page/wallet_personalize_intro_page.dart';
import 'page/wallet_personalize_success_page.dart';
import 'widget/mock_digid_screen.dart';

class WalletPersonalizeScreen extends StatelessWidget {
  const WalletPersonalizeScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      restorationId: 'wallet_personalize_scaffold',
      appBar: AppBar(
        title: Text(AppLocalizations.of(context).walletPersonalizeScreenTitle),
      ),
      body: WillPopScope(
        onWillPop: () async => false,
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
    return BlocBuilder<WalletPersonalizeBloc, WalletPersonalizeState>(
      buildWhen: (prev, current) => prev.stepperProgress != current.stepperProgress,
      builder: (context, state) => AnimatedLinearProgressIndicator(progress: state.stepperProgress),
    );
  }

  Widget _buildPage() {
    return BlocConsumer<WalletPersonalizeBloc, WalletPersonalizeState>(
      listenWhen: (prev, current) => current is WalletPersonalizeLoadingPid,
      listener: (context, state) => _loginWithDigid(context),
      builder: (context, state) {
        Widget? result;
        if (state is WalletPersonalizeInitial) result = _buildWalletIntroPage(context);
        if (state is WalletPersonalizeLoadingPid) result = _buildLoading(context);
        if (state is WalletPersonalizeCheckData) result = _buildCheckDataOfferingPage(context, state);
        if (state is WalletPersonalizeSuccess) result = _buildSuccessPage(context, state);
        if (state is WalletPersonalizeFailure) result = _buildErrorPage(context);
        if (result == null) throw UnsupportedError('Unknown state: $state');
        return FakePagingAnimatedSwitcher(animateBackwards: state.didGoBack, child: result);
      },
    );
  }

  Widget _buildCheckDataOfferingPage(BuildContext context, WalletPersonalizeCheckData state) {
    return WalletPersonalizeCheckDataOfferingPage(
      onAccept: () => context.read<WalletPersonalizeBloc>().add(WalletPersonalizeOfferingAccepted(state.pidCard)),
      attributes: state.pidCard.attributes,
    );
  }

  Widget _buildLoading(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        Text(
          locale.walletPersonalizeScreenLoadingTitle,
          style: Theme.of(context).textTheme.headline4,
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 8),
        Text(
          locale.walletPersonalizeScreenLoadingSubtitle,
          style: Theme.of(context).textTheme.bodyText1,
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 24),
        const CenteredLoadingIndicator(),
      ],
    );
  }

  Widget _buildWalletIntroPage(BuildContext context) {
    return WalletPersonalizeIntroPage(
      onLoginWithDigidPressed: () {
        context.read<WalletPersonalizeBloc>().add(WalletPersonalizeLoginWithDigidClicked());
      },
      onNoDigidPressed: () {
        PlaceholderScreen.show(context, AppLocalizations.of(context).walletPersonalizeIntroPageNoDigidCta);
      },
    );
  }

  void _loginWithDigid(BuildContext context) async {
    final bloc = context.read<WalletPersonalizeBloc>();
    await MockDigidScreen.show(context);
    await Future.delayed(kDefaultMockDelay);
    bloc.add(WalletPersonalizeLoginWithDigidSucceeded());
  }

  Widget _buildSuccessPage(BuildContext context, WalletPersonalizeSuccess state) {
    return WalletPersonalizeSuccessPage(
      onContinuePressed: () => Navigator.pushReplacementNamed(context, WalletRoutes.homeRoute),
      cardFront: state.cardFront,
    );
  }

  Widget _buildErrorPage(BuildContext context) {
    return Center(
      child: IconButton(
        iconSize: 64,
        onPressed: () => context.read<WalletPersonalizeBloc>().add(WalletPersonalizeOnRetryClicked()),
        icon: Icon(
          Icons.error,
          color: Theme.of(context).errorColor,
        ),
      ),
    );
  }
}
