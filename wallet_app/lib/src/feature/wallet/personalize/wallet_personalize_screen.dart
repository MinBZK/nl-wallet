import 'package:collection/collection.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:url_launcher/url_launcher_string.dart';

import '../../../../environment.dart';
import '../../../domain/model/attribute/ui_attribute.dart';
import '../../../domain/usecase/card/get_pid_issuance_response_usecase.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/mapper/pid/pid_attribute_mapper.dart';
import '../../../wallet_constants.dart';
import '../../common/page/flow_terminal_page.dart';
import '../../common/page/generic_loading_page.dart';
import '../../common/sheet/confirm_action_sheet.dart';
import '../../common/widget/animated_linear_progress_indicator.dart';
import '../../common/widget/button/animated_visibility_back_button.dart';
import '../../common/widget/fake_paging_animated_switcher.dart';
import '../../digid_help/digid_help_screen.dart';
import '../../home/home_screen.dart';
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
        title: Text(context.l10n.walletPersonalizeScreenTitle),
      ),
      body: PopScope(
        canPop: false,
        onPopInvoked: (didPop) {
          if (didPop) {
            return;
          }
          if (context.bloc.state.canGoBack) {
            context.bloc.add(WalletPersonalizeBackPressed());
          } else {
            _showExitSheet(context);
          }
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
          onPressed: () => context.bloc.add(WalletPersonalizeBackPressed()),
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
      listener: (context, state) {
        _closeOpenDialogs(context);
        if (state is WalletPersonalizeConnectDigid) _loginWithDigid(context, state.authUrl);
      },
      builder: (context, state) {
        Widget result = switch (state) {
          WalletPersonalizeInitial() => _buildWalletIntroPage(context),
          WalletPersonalizeLoadingIssuanceUrl() => _buildAuthenticatingWithDigid(context),
          WalletPersonalizeConnectDigid() => _buildAuthenticatingWithDigid(context),
          WalletPersonalizeAuthenticating() => _buildAuthenticatingWithDigid(context),
          WalletPersonalizeLoadInProgress() => _buildLoading(context),
          WalletPersonalizeCheckData() => _buildCheckDataOfferingPage(context, state),
          WalletPersonalizeConfirmPin() => _buildConfirmPinPage(context, state),
          WalletPersonalizeSuccess() => _buildSuccessPage(context, state),
          WalletPersonalizeFailure() => _buildErrorPage(context),
          WalletPersonalizeDigidCancelled() => _buildDigidCancelledPage(context),
          WalletPersonalizeDigidFailure() => _buildDigidErrorPage(context),
        };
        return FakePagingAnimatedSwitcher(animateBackwards: state.didGoBack, child: result);
      },
    );
  }

  /// Closes any dialogs opened on top of this [WalletPersonalizeScreen], ignored if none exist.
  void _closeOpenDialogs(BuildContext context) => Navigator.popUntil(context, (route) => route is! DialogRoute);

  Widget _buildCheckDataOfferingPage(BuildContext context, WalletPersonalizeCheckData state) {
    /// Note that mapping occurs in the UI layer since we need a fresh context (with l10n).
    List<UiAttribute> attributes = [];
    try {
      attributes = context.read<PidAttributeMapper>().map(context, state.availableAttributes);
    } catch (ex) {
      Fimber.e('Failed to map pid attributes to expected preview attributes', ex: ex);
      context.bloc.add(const WalletPersonalizeLoginWithDigidFailed());
    }
    return WalletPersonalizeCheckDataOfferingPage(
      key: const Key('personalizePidPreviewPage'),
      onAcceptPressed: () => context.bloc.add(WalletPersonalizeOfferingAccepted(state.availableAttributes)),
      onRejectPressed: () => context.bloc.add(WalletPersonalizeOfferingRejected()),
      attributes: attributes,
    );
  }

  Widget _buildLoading(BuildContext context, {VoidCallback? onCancel}) {
    return GenericLoadingPage(
      key: const Key('personalizeLoadingPage'),
      title: context.l10n.walletPersonalizeScreenLoadingTitle,
      description: context.l10n.walletPersonalizeScreenLoadingSubtitle,
      onCancel: onCancel,
    );
  }

  Widget _buildAuthenticatingWithDigid(BuildContext context) {
    return GenericLoadingPage(
      title: context.l10n.walletPersonalizeScreenDigidLoadingTitle,
      description: context.l10n.walletPersonalizeScreenDigidLoadingSubtitle,
      cancelCta: context.l10n.walletPersonalizeScreenDigidLoadingStopCta,
      onCancel: () async {
        final bloc = context.bloc;
        final cancelled = await _showStopDigidLoginDialog(context);
        if (cancelled) bloc.add(const WalletPersonalizeLoginWithDigidFailed(cancelledByUser: true));
      },
    );
  }

  Future<bool> _showStopDigidLoginDialog(BuildContext context) async {
    /// This check helps avoid a race condition where the dialog is opened after the state change, meaning it would
    /// not be closed by [_closeOpenDialogs] as intended.
    final isAuthenticating = context.bloc.state is WalletPersonalizeAuthenticating;
    final isConnectingToDigid = context.bloc.state is WalletPersonalizeConnectDigid;
    final shouldShowDialog = isAuthenticating || isConnectingToDigid;
    if (!shouldShowDialog) return false;

    final result = await showDialog<bool?>(
      context: context,
      builder: (BuildContext context) {
        return AlertDialog(
          title: Text(context.l10n.walletPersonalizeScreenStopDigidDialogTitle),
          content: Text(context.l10n.walletPersonalizeScreenStopDigidDialogSubtitle),
          actions: <Widget>[
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: Text(context.l10n.walletPersonalizeScreenStopDigidDialogNegativeCta),
            ),
            TextButton(
              style: Theme.of(context)
                  .textButtonTheme
                  .style
                  ?.copyWith(foregroundColor: MaterialStatePropertyAll(Theme.of(context).colorScheme.error)),
              onPressed: () => Navigator.pop(context, true),
              child: Text(context.l10n.walletPersonalizeScreenStopDigidDialogPositiveCta),
            ),
          ],
        );
      },
    );
    return result == true;
  }

  Widget _buildWalletIntroPage(BuildContext context) {
    return WalletPersonalizeIntroPage(
      key: const Key('personalizeInformPage'),
      onLoginWithDigidPressed: () => context.bloc.add(WalletPersonalizeLoginWithDigidClicked()),
      onNoDigidPressed: () => WalletPersonalizeNoDigidScreen.show(context),
    );
  }

  void _loginWithDigid(BuildContext context, String authUrl) async {
    if (Environment.mockRepositories && !Environment.isTest) {
      await _performMockDigidLogin(context);
    } else {
      try {
        launchUrlString(authUrl, mode: LaunchMode.externalApplication);
      } catch (ex) {
        Fimber.e('Failed to open auth url: $authUrl', ex: ex);
        context.bloc.add(const WalletPersonalizeLoginWithDigidFailed());
      }
    }
  }

  Future<void> _performMockDigidLogin(BuildContext context) async {
    final bloc = context.bloc;
    final getPidIssuanceResponseUseCase = context.read<GetPidIssuanceResponseUseCase>();
    // Perform the mock DigiD flow
    final loginSucceeded = (await MockDigidScreen.mockLogin(context)) == true;
    await Future.delayed(kDefaultMockDelay);
    if (loginSucceeded) {
      final mockPidIssuance = await getPidIssuanceResponseUseCase.invoke();
      final mockPidAttributes = mockPidIssuance.cards.map((e) => e.attributes).flattened.toList();
      bloc.add(WalletPersonalizeLoginWithDigidSucceeded(mockPidAttributes));
    } else {
      bloc.add(const WalletPersonalizeLoginWithDigidFailed());
    }
  }

  Widget _buildSuccessPage(BuildContext context, WalletPersonalizeSuccess state) {
    return WalletPersonalizeSuccessPage(
      key: const Key('personalizeSuccessPage'),
      onContinuePressed: () => HomeScreen.show(context, cards: state.addedCards),
      cards: state.addedCards,
    );
  }

  Widget _buildErrorPage(BuildContext context) {
    return FlowTerminalPage(
      icon: Icons.not_interested,
      iconColor: context.theme.primaryColorDark,
      title: context.l10n.walletPersonalizeScreenErrorTitle,
      description: context.l10n.walletPersonalizeScreenErrorDescription,
      primaryButtonCta: context.l10n.walletPersonalizeScreenErrorRetryCta,
      onPrimaryPressed: () => context.bloc.add(WalletPersonalizeRetryPressed()),
    );
  }

  Widget _buildDigidCancelledPage(BuildContext context) {
    return WalletPersonalizeDigidErrorPage(
      title: context.l10n.walletPersonalizeDigidCancelledPageTitle,
      description: context.l10n.walletPersonalizeDigidCancelledPageDescription,
      onRetryPressed: () => context.bloc.add(WalletPersonalizeLoginWithDigidClicked()),
      onHelpPressed: () => DigidHelpScreen.show(context, title: context.l10n.walletPersonalizeScreenTitle),
    );
  }

  Widget _buildDigidErrorPage(BuildContext context) {
    return WalletPersonalizeDigidErrorPage(
      title: context.l10n.walletPersonalizeDigidErrorPageTitle,
      description: context.l10n.walletPersonalizeDigidErrorPageDescription,
      onRetryPressed: () => context.bloc.add(WalletPersonalizeLoginWithDigidClicked()),
      onHelpPressed: () => DigidHelpScreen.show(context, title: context.l10n.walletPersonalizeScreenTitle),
    );
  }

  ///FIXME: Temporary solution to make sure the user doesn't accidentally cancel the creation flow but can still exit.
  void _showExitSheet(BuildContext context) async {
    final confirmed = await ConfirmActionSheet.show(
      context,
      title: context.l10n.walletPersonalizeScreenExitSheetTitle,
      description: context.l10n.walletPersonalizeScreenExitSheetDescription,
      cancelButtonText: context.l10n.walletPersonalizeScreenExitSheetCancelCta,
      confirmButtonText: context.l10n.walletPersonalizeScreenExitSheetConfirmCta,
      confirmButtonColor: context.colorScheme.error,
    );
    if (confirmed && context.mounted) Navigator.pop(context);
  }

  Widget _buildConfirmPinPage(BuildContext context, WalletPersonalizeConfirmPin state) {
    return WalletPersonalizeConfirmPinPage(
      onPidAccepted: () => context.bloc.add(WalletPersonalizePinConfirmed()),
    );
  }
}

extension _WalletPersonalizeScreenExtension on BuildContext {
  WalletPersonalizeBloc get bloc => read<WalletPersonalizeBloc>();
}
