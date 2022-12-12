import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../wallet_constants.dart';
import '../../../wallet_routes.dart';
import '../../common/widget/animated_linear_progress_indicator.dart';
import '../../common/widget/animated_visibility_back_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/fake_paging_animated_switcher.dart';
import '../../common/widget/placeholder_screen.dart';
import '../../wallet/personalize/bloc/wallet_personalize_bloc.dart';
import 'page/wallet_personalize_check_data_offering_page.dart';
import 'page/wallet_personalize_intro_page.dart';
import 'page/wallet_personalize_loading_photo_page.dart';
import 'page/wallet_personalize_photo_added_page.dart';
import 'page/wallet_personalize_scan_id_intro_page.dart';
import 'page/wallet_personalize_scan_id_page.dart';
import 'page/wallet_personalize_success_page.dart';
import 'widget/mock_digid_screen.dart';

class WalletPersonalizeScreen extends StatelessWidget {
  const WalletPersonalizeScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      restorationId: 'wallet_personalize_scaffold',
      appBar: AppBar(
        leading: _buildBackButton(context),
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

  Widget _buildBackButton(BuildContext context) {
    return BlocBuilder<WalletPersonalizeBloc, WalletPersonalizeState>(
      builder: (context, state) {
        return AnimatedVisibilityBackButton(
          visible: state.canGoBack,
          onPressed: () => context.read<WalletPersonalizeBloc>().add(WalletPersonalizeOnBackPressed()),
        );
      },
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
        if (state is WalletPersonalizeScanIdIntro) result = _buildScanIdIntroPage(context, state);
        if (state is WalletPersonalizeScanId) result = _buildScanIdPage(context, state);
        if (state is WalletPersonalizeLoadingPhoto) result = _buildLoadingPhotoPage(context, state);
        if (state is WalletPersonalizePhotoAdded) result = _buildPhotoAddedPage(context, state);
        if (state is WalletPersonalizeSuccess) result = _buildSuccessPage(context, state);
        if (state is WalletPersonalizeFailure) result = _buildErrorPage(context);
        if (result == null) throw UnsupportedError('Unknown state: $state');
        return FakePagingAnimatedSwitcher(animateBackwards: state.didGoBack, child: result);
      },
    );
  }

  Widget _buildCheckDataOfferingPage(BuildContext context, WalletPersonalizeCheckData state) {
    return WalletPersonalizeCheckDataOfferingPage(
      onAccept: () => context.read<WalletPersonalizeBloc>().add(WalletPersonalizeOfferingVerified()),
      attributes: state.availableAttributes,
      name: state.firstNames,
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
      onContinuePressed: () => Navigator.restorablePushReplacementNamed(context, WalletRoutes.homeRoute),
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

  Widget _buildScanIdIntroPage(BuildContext context, WalletPersonalizeScanIdIntro state) {
    return WalletPersonalizeScanIdIntroPage(
      onStartScanPressed: () => context.read<WalletPersonalizeBloc>().add(WalletPersonalizeScanInitiated()),
    );
  }

  Widget _buildScanIdPage(BuildContext context, WalletPersonalizeScanId state) {
    return WalletPersonalizeScanIdPage(
      onNfcDetected: () => context.read<WalletPersonalizeBloc>().add(WalletPersonalizeScanEvent()),
    );
  }

  Widget _buildLoadingPhotoPage(BuildContext context, WalletPersonalizeLoadingPhoto state) {
    return WalletPersonalizeLoadingPhotoPage(mockDelay: state.mockedScanDuration);
  }

  Widget _buildPhotoAddedPage(BuildContext context, WalletPersonalizePhotoAdded state) {
    return WalletPersonalizePhotoAddedPage(
      photo: state.photo,
      onNextPressed: () => context.read<WalletPersonalizeBloc>().add(WalletPersonalizePhotoApproved()),
    );
  }
}
