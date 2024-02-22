import 'package:collection/collection.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:url_launcher/url_launcher_string.dart';
import 'package:wallet_mock/mock.dart';

import '../../../../environment.dart';
import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/attribute/ui_attribute.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/mapper/card/attribute/card_attribute_mapper.dart';
import '../../../util/mapper/mapper.dart';
import '../../../util/mapper/pid/pid_attribute_mapper.dart';
import '../../../wallet_constants.dart';
import '../../../wallet_core/typed/typed_wallet_core.dart';
import '../../common/page/generic_loading_page.dart';
import '../../common/page/legacy_terminal_page.dart';
import '../../common/sheet/confirm_action_sheet.dart';
import '../../common/widget/button/animated_visibility_back_button.dart';
import '../../common/widget/fake_paging_animated_switcher.dart';
import '../../common/widget/wallet_app_bar.dart';
import '../../dashboard/dashboard_screen.dart';
import '../../digid_help/digid_help_screen.dart';
import '../../error/error_page.dart';
import '../../mock_digid/mock_digid_screen.dart';
import '../../wallet/personalize/bloc/wallet_personalize_bloc.dart';
import 'page/wallet_personalize_check_data_offering_page.dart';
import 'page/wallet_personalize_confirm_pin_page.dart';
import 'page/wallet_personalize_digid_error_page.dart';
import 'page/wallet_personalize_intro_page.dart';
import 'page/wallet_personalize_success_page.dart';
import 'wallet_personalize_no_digid_screen.dart';

class WalletPersonalizeScreen extends StatelessWidget {
  const WalletPersonalizeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      restorationId: 'wallet_personalize_scaffold',
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
        child: SafeArea(
          child: _buildPage(),
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

  Widget _buildPage() {
    return BlocConsumer<WalletPersonalizeBloc, WalletPersonalizeState>(
      listener: (context, state) {
        _closeOpenDialogs(context);
        if (state is WalletPersonalizeConnectDigid) _loginWithDigid(context, state.authUrl);
      },
      builder: (context, state) {
        Widget result = switch (state) {
          WalletPersonalizeInitial() => _buildWalletIntroPage(context, state),
          WalletPersonalizeLoadingIssuanceUrl() =>
            _buildAuthenticatingWithDigid(context, progress: state.stepperProgress),
          WalletPersonalizeConnectDigid() => _buildAuthenticatingWithDigid(context, progress: state.stepperProgress),
          WalletPersonalizeAuthenticating() => _buildAuthenticatingWithDigid(context, progress: state.stepperProgress),
          WalletPersonalizeLoadInProgress() => _buildLoading(context, progress: state.stepperProgress),
          WalletPersonalizeCheckData() => _buildCheckDataOfferingPage(context, state),
          WalletPersonalizeConfirmPin() => _buildConfirmPinPage(context, state),
          WalletPersonalizeSuccess() => _buildSuccessPage(context, state),
          WalletPersonalizeFailure() => _buildErrorPage(context),
          WalletPersonalizeDigidCancelled() => _buildDigidCancelledPage(context),
          WalletPersonalizeDigidFailure() => _buildDigidErrorPage(context),
          WalletPersonalizeNetworkError() => _buildNetworkError(context, state),
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
      progress: state.stepperProgress,
    );
  }

  Widget _buildLoading(BuildContext context, {VoidCallback? onCancel, double? progress}) {
    return GenericLoadingPage(
      title: context.l10n.walletPersonalizeScreenLoadingTitle,
      description: context.l10n.walletPersonalizeScreenLoadingSubtitle,
      onCancel: onCancel,
      appBar: WalletAppBar(progress: progress),
    );
  }

  Widget _buildAuthenticatingWithDigid(BuildContext context, {double? progress}) {
    return GenericLoadingPage(
      key: const Key('personalizeAuthenticatingWithDigidPage'),
      title: context.l10n.walletPersonalizeScreenDigidLoadingTitle,
      description: context.l10n.walletPersonalizeScreenDigidLoadingSubtitle,
      cancelCta: context.l10n.walletPersonalizeScreenDigidLoadingStopCta,
      appBar: WalletAppBar(progress: progress),
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
                  ?.copyWith(foregroundColor: MaterialStatePropertyAll(context.colorScheme.error)),
              onPressed: () => Navigator.pop(context, true),
              child: Text(context.l10n.walletPersonalizeScreenStopDigidDialogPositiveCta),
            ),
          ],
        );
      },
    );
    return result == true;
  }

  Widget _buildWalletIntroPage(BuildContext context, WalletPersonalizeInitial state) {
    return WalletPersonalizeIntroPage(
      key: const Key('personalizeInformPage'),
      onLoginWithDigidPressed: () => context.bloc.add(WalletPersonalizeLoginWithDigidClicked()),
      onNoDigidPressed: () => WalletPersonalizeNoDigidScreen.show(context),
      progress: state.stepperProgress,
    );
  }

  void _loginWithDigid(BuildContext context, String authUrl) async {
    if (authUrl == kMockPidIssuanceRedirectUri && !Environment.isTest) {
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

  /// Initiate the mock digid login and notify the bloc about the result
  /// FIXME: The wallet_app is still quite aware of the mock here, room for improvement.
  Future<void> _performMockDigidLogin(BuildContext context) async {
    final bloc = context.bloc;
    final walletCore = context.read<TypedWalletCore>();
    final Mapper<CardAttributeWithDocType, DataAttribute> attributeMapper = context.read();

    // Perform the mock DigiD flow
    final loginSucceeded = (await MockDigidScreen.mockLogin(context)) == true;
    await Future.delayed(kDefaultMockDelay);
    if (loginSucceeded) {
      // FIXME: Still taking some shortcuts here that require knowledge about the mock
      final cards = await walletCore.continuePidIssuance(kMockPidIssuanceRedirectUri);
      final mockPidCardAttributes =
          cards.map((card) => card.attributes.map((e) => CardAttributeWithDocType(card.docType, e))).flattened.toList();
      bloc.add(WalletPersonalizeLoginWithDigidSucceeded(attributeMapper.mapList(mockPidCardAttributes)));
    } else {
      bloc.add(const WalletPersonalizeLoginWithDigidFailed());
    }
  }

  Widget _buildSuccessPage(BuildContext context, WalletPersonalizeSuccess state) {
    return Scaffold(
      appBar: WalletAppBar(
        progress: state.stepperProgress,
      ),
      body: WalletPersonalizeSuccessPage(
        key: const Key('personalizeSuccessPage'),
        onContinuePressed: () => DashboardScreen.show(context, cards: state.addedCards),
        cards: state.addedCards,
      ),
    );
  }

  Widget _buildErrorPage(BuildContext context) {
    return Scaffold(
      appBar: const WalletAppBar(
        progress: 0.0,
      ),
      body: LegacyTerminalPage(
        icon: Icons.not_interested,
        iconColor: context.theme.primaryColorDark,
        title: context.l10n.walletPersonalizeScreenErrorTitle,
        description: context.l10n.walletPersonalizeScreenErrorDescription,
        primaryButtonCta: context.l10n.walletPersonalizeScreenErrorRetryCta,
        onPrimaryPressed: () => context.bloc.add(WalletPersonalizeRetryPressed()),
      ),
    );
  }

  Widget _buildDigidCancelledPage(BuildContext context) {
    return Scaffold(
      appBar: const WalletAppBar(
        progress: 0.0,
      ),
      body: WalletPersonalizeDigidErrorPage(
        title: context.l10n.walletPersonalizeDigidCancelledPageTitle,
        description: context.l10n.walletPersonalizeDigidCancelledPageDescription,
        onRetryPressed: () => context.bloc.add(WalletPersonalizeLoginWithDigidClicked()),
        onHelpPressed: () => DigidHelpScreen.show(context, title: context.l10n.walletPersonalizeScreenTitle),
      ),
    );
  }

  Widget _buildDigidErrorPage(BuildContext context) {
    return Scaffold(
      appBar: const WalletAppBar(
        progress: 0.0,
      ),
      body: WalletPersonalizeDigidErrorPage(
        title: context.l10n.walletPersonalizeDigidErrorPageTitle,
        description: context.l10n.walletPersonalizeDigidErrorPageDescription,
        onRetryPressed: () => context.bloc.add(WalletPersonalizeLoginWithDigidClicked()),
        onHelpPressed: () => DigidHelpScreen.show(context, title: context.l10n.walletPersonalizeScreenTitle),
      ),
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
    return Scaffold(
      appBar: WalletAppBar(
        progress: state.stepperProgress,
        leading: _buildBackButton(context),
      ),
      body: WalletPersonalizeConfirmPinPage(
        onPidAccepted: (_) => context.bloc.add(WalletPersonalizePinConfirmed()),
      ),
    );
  }

  Widget _buildNetworkError(BuildContext context, WalletPersonalizeNetworkError state) {
    if (state.hasInternet) {
      return Scaffold(
        appBar: WalletAppBar(progress: state.stepperProgress),
        body: ErrorPage.network(
          context,
          onPrimaryActionPressed: () => context.bloc.add(WalletPersonalizeRetryPressed()),
        ),
      );
    } else {
      return Scaffold(
        appBar: WalletAppBar(progress: state.stepperProgress),
        body: ErrorPage.noInternet(
          context,
          onPrimaryActionPressed: () => context.bloc.add(WalletPersonalizeRetryPressed()),
        ),
      );
    }
  }
}

extension _WalletPersonalizeScreenExtension on BuildContext {
  WalletPersonalizeBloc get bloc => read<WalletPersonalizeBloc>();
}
