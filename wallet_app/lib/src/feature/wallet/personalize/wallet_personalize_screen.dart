import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../util/mapper/pid_attributes_mapper.dart';
import '../../../wallet_constants.dart';
import '../../../wallet_routes.dart';
import '../../common/widget/animated_linear_progress_indicator.dart';
import '../../common/widget/button/animated_visibility_back_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/confirm_action_sheet.dart';
import '../../common/widget/fake_paging_animated_switcher.dart';
import '../../common/widget/placeholder_screen.dart';
import '../../mock_digid/mock_digid_screen.dart';
import '../../wallet/personalize/bloc/wallet_personalize_bloc.dart';
import 'page/wallet_personalize_check_data_offering_page.dart';
import 'page/wallet_personalize_confirm_pin_page.dart';
import 'page/wallet_personalize_digid_error_page.dart';
import 'page/wallet_personalize_intro_page.dart';
import 'wallet_personalize_no_digid_screen.dart';
import 'page/wallet_personalize_success_page.dart';

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
        onWillPop: () async {
          if (context.bloc.state.canGoBack) {
            context.bloc.add(WalletPersonalizeOnBackPressed());
          } else {
            return _showExitSheet(context);
          }
          return false;
        },
        child: Column(
          children: [
            _buildStepper(),
            Expanded(
              child: SafeArea(
                child: _buildPage(),
              ),
            ),
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
          onPressed: () => context.bloc.add(WalletPersonalizeOnBackPressed()),
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
        if (state is WalletPersonalizeLoadInProgress) result = _buildLoading(context);
        if (state is WalletPersonalizeCheckData) result = _buildCheckDataOfferingPage(context, state);
        if (state is WalletPersonalizeConfirmPin) result = _buildConfirmPinPage(context, state);
        if (state is WalletPersonalizeSuccess) result = _buildSuccessPage(context, state);
        if (state is WalletPersonalizeFailure) result = _buildErrorPage(context);
        if (state is WalletPersonalizeDigidFailure) result = _buildDigidErrorPage(context);
        if (result == null) throw UnsupportedError('Unknown state: $state');
        return FakePagingAnimatedSwitcher(animateBackwards: state.didGoBack, child: result);
      },
    );
  }

  Widget _buildCheckDataOfferingPage(BuildContext context, WalletPersonalizeCheckData state) {
    return WalletPersonalizeCheckDataOfferingPage(
      onAcceptPressed: () => context.bloc.add(WalletPersonalizeOfferingVerified()),
      attributes: PidAttributeMapper.map(state.availableAttributes),
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
          style: Theme.of(context).textTheme.headlineMedium,
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 8),
        Text(
          locale.walletPersonalizeScreenLoadingSubtitle,
          style: Theme.of(context).textTheme.bodyLarge,
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 24),
        const CenteredLoadingIndicator(),
      ],
    );
  }

  Widget _buildWalletIntroPage(BuildContext context) {
    return WalletPersonalizeIntroPage(
      onLoginWithDigidPressed: () => context.bloc.add(WalletPersonalizeLoginWithDigidClicked()),
      onNoDigidPressed: () => WalletPersonalizeNoDigidScreen.show(context),
    );
  }

  void _loginWithDigid(BuildContext context) async {
    final bloc = context.bloc;
    final loginSucceeded = (await MockDigidScreen.mockLogin(context)) == true;
    await Future.delayed(kDefaultMockDelay);
    if (loginSucceeded) {
      bloc.add(WalletPersonalizeLoginWithDigidSucceeded());
    } else {
      bloc.add(WalletPersonalizeLoginWithDigidFailed());
    }
  }

  Widget _buildSuccessPage(BuildContext context, WalletPersonalizeSuccess state) {
    return WalletPersonalizeSuccessPage(
      onContinuePressed: () => Navigator.restorablePushReplacementNamed(context, WalletRoutes.homeRoute),
      cards: state.cardFronts,
    );
  }

  Widget _buildErrorPage(BuildContext context) {
    return Center(
      child: IconButton(
        iconSize: 64,
        onPressed: () => context.bloc.add(WalletPersonalizeOnRetryClicked()),
        icon: Icon(
          Icons.error,
          color: Theme.of(context).colorScheme.error,
        ),
      ),
    );
  }

  Widget _buildDigidErrorPage(BuildContext context) {
    return WalletPersonalizeDigidErrorPage(
      onRetryPressed: () => context.bloc.add(WalletPersonalizeLoginWithDigidClicked()),
      onHelpPressed: () => PlaceholderScreen.show(context),
    );
  }

  ///FIXME: Temporary solution to make sure the user doesn't accidentally cancel the creation flow but can still exit.
  Future<bool> _showExitSheet(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return ConfirmActionSheet.show(
      context,
      title: locale.walletPersonalizeScreenExitSheetTitle,
      description: locale.walletPersonalizeScreenExitSheetDescription,
      cancelButtonText: locale.walletPersonalizeScreenExitSheetCancelCta,
      confirmButtonText: locale.walletPersonalizeScreenExitSheetConfirmCta,
      confirmButtonColor: Theme.of(context).colorScheme.error,
    );
  }

  Widget _buildConfirmPinPage(BuildContext context, WalletPersonalizeConfirmPin state) {
    return WalletPersonalizeConfirmPinPage(
      onPinValidated: () => context.bloc.add(WalletPersonalizePinConfirmed()),
    );
  }
}

extension _WalletPersonalizeScreenExtension on BuildContext {
  WalletPersonalizeBloc get bloc => read<WalletPersonalizeBloc>();
}
