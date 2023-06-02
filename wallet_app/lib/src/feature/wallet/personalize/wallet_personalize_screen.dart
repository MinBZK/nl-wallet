import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:url_launcher/url_launcher_string.dart';

import '../../../../environment.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../util/mapper/pid_attributes_mapper.dart';
import '../../../wallet_constants.dart';
import '../../common/widget/animated_linear_progress_indicator.dart';
import '../../common/widget/button/animated_visibility_back_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/confirm_action_sheet.dart';
import '../../common/widget/fake_paging_animated_switcher.dart';
import '../../common/widget/flow_terminal_page.dart';
import '../../common/widget/placeholder_screen.dart';
import '../../mock_digid/mock_digid_screen.dart';
import '../../wallet/personalize/bloc/wallet_personalize_bloc.dart';
import 'page/wallet_personalize_check_data_offering_page.dart';
import 'page/wallet_personalize_confirm_pin_page.dart';
import 'page/wallet_personalize_digid_error_page.dart';
import 'page/wallet_personalize_intro_page.dart';
import 'page/wallet_personalize_success_page.dart';
import 'wallet_personalize_no_digid_screen.dart';

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
      listenWhen: (prev, current) => current is WalletPersonalizeConnectDigid,
      listener: (context, state) => _loginWithDigid(context, (state as WalletPersonalizeConnectDigid).authUrl),
      builder: (context, state) {
        Widget? result;
        if (state is WalletPersonalizeInitial) result = _buildWalletIntroPage(context);
        if (state is WalletPersonalizeConnectDigid) result = _buildAuthenticatingWithDigid(context);
        if (state is WalletPersonalizeAuthenticating) result = _buildAuthenticatingWithDigid(context);
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

  Widget _buildLoading(BuildContext context, {VoidCallback? onCancel}) {
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
        if (onCancel != null) ...[
          const SizedBox(height: 48),
          TextButton(
            onPressed: onCancel,
            child: Text(locale.generalCancelCta),
          ),
        ],
      ],
    );
  }

  Widget _buildAuthenticatingWithDigid(BuildContext context) {
    return _buildLoading(context, onCancel: () => context.bloc.add(WalletPersonalizeLoginWithDigidFailed()));
  }

  Widget _buildWalletIntroPage(BuildContext context) {
    return WalletPersonalizeIntroPage(
      onLoginWithDigidPressed: () => context.bloc.add(WalletPersonalizeLoginWithDigidClicked()),
      onNoDigidPressed: () => WalletPersonalizeNoDigidScreen.show(context),
    );
  }

  void _loginWithDigid(BuildContext context, String authUrl) async {
    final bloc = context.bloc;
    if (Environment.mockRepositories) {
      // Perform the mock DigiD flow
      final loginSucceeded = (await MockDigidScreen.mockLogin(context)) == true;
      await Future.delayed(kDefaultMockDelay);
      if (loginSucceeded) {
        bloc.add(WalletPersonalizeLoginWithDigidSucceeded());
      } else {
        bloc.add(WalletPersonalizeLoginWithDigidFailed());
      }
    } else {
      try {
        launchUrlString(authUrl, mode: LaunchMode.externalApplication);
      } catch (ex) {
        Fimber.e('Failed to open auth url: $authUrl', ex: ex);
        bloc.add(WalletPersonalizeLoginWithDigidFailed());
      }
    }
  }

  Widget _buildSuccessPage(BuildContext context, WalletPersonalizeSuccess state) {
    return WalletPersonalizeSuccessPage(
      onContinuePressed: () => Navigator.restorablePushReplacementNamed(context, WalletRoutes.homeRoute),
      cards: state.cardFronts,
    );
  }

  Widget _buildErrorPage(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return FlowTerminalPage(
      icon: Icons.not_interested,
      iconColor: Theme.of(context).primaryColorDark,
      title: locale.walletPersonalizeScreenErrorTitle,
      description: locale.walletPersonalizeScreenErrorDescription,
      closeButtonCta: locale.walletPersonalizeScreenErrorRetryCta,
      onClosePressed: () => context.bloc.add(WalletPersonalizeOnRetryClicked()),
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
