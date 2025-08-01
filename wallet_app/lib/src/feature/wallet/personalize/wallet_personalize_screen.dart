import 'dart:io';

import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:url_launcher/url_launcher_string.dart';

import '../../../../environment.dart';
import '../../../data/service/navigation_service.dart';
import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/flow_progress.dart';
import '../../../domain/model/navigation/navigation_request.dart';
import '../../../domain/model/result/application_error.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/launch_util.dart';
import '../../../wallet_assets.dart';
import '../../../wallet_constants.dart';
import '../../common/dialog/stop_digid_login_dialog.dart';
import '../../common/page/generic_loading_page.dart';
import '../../common/page/terminal_page.dart';
import '../../common/sheet/confirm_action_sheet.dart';
import '../../common/widget/button/animated_visibility_back_button.dart';
import '../../common/widget/fade_in_at_offset.dart';
import '../../common/widget/fake_paging_animated_switcher.dart';
import '../../common/widget/loading_indicator.dart';
import '../../common/widget/page_illustration.dart';
import '../../common/widget/svg_or_image.dart';
import '../../common/widget/text/title_text.dart';
import '../../common/widget/wallet_app_bar.dart';
import '../../dashboard/dashboard_screen.dart';
import '../../error/error_page.dart';
import '../../mock_digid/mock_digid_screen.dart';
import '../../wallet/personalize/bloc/wallet_personalize_bloc.dart';
import 'page/wallet_personalize_check_data_offering_page.dart';
import 'page/wallet_personalize_confirm_pin_page.dart';
import 'page/wallet_personalize_intro_page.dart';
import 'page/wallet_personalize_success_page.dart';

class WalletPersonalizeScreen extends StatelessWidget {
  const WalletPersonalizeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      restorationId: 'wallet_personalize_scaffold',
      body: PopScope(
        canPop: false,
        onPopInvokedWithResult: (didPop, result) {
          if (didPop) return;

          if (context.bloc.state.canGoBack) {
            context.bloc.add(WalletPersonalizeBackPressed());
          } else {
            _showExitSheet(context);
          }
        },
        child: ScrollOffsetProvider(
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
        context.read<ScrollOffset>().reset(); // Reset provided scrollOffset between pages
        _closeOpenDialogs(context); // Make sure the StopDigidLoginDialog is dismissed on state changes.
        if (state is WalletPersonalizeConnectDigid) _loginWithDigid(context, state.authUrl);
      },
      builder: (context, state) {
        final Widget result = switch (state) {
          WalletPersonalizeInitial() => _buildWalletIntroPage(context, state),
          WalletPersonalizeLoadingIssuanceUrl() => _buildAuthenticatingWithDigid(
              context,
              progress: state.stepperProgress,
              stage: DigiDAuthStage.fetchingAuthUrl,
            ),
          WalletPersonalizeConnectDigid() => _buildAuthenticatingWithDigid(
              context,
              progress: state.stepperProgress,
              stage: DigiDAuthStage.awaitingUserAction,
            ),
          WalletPersonalizeAuthenticating() => _buildAuthenticatingWithDigid(
              context,
              progress: state.stepperProgress,
              stage: DigiDAuthStage.processingResult,
            ),
          WalletPersonalizeLoadInProgress() => _buildLoading(context, progress: state.stepperProgress),
          WalletPersonalizeCheckData() => _buildCheckDataOfferingPage(context, state),
          WalletPersonalizeConfirmPin() => _buildConfirmPinPage(context, state),
          WalletPersonalizeSuccess() => _buildSuccessPage(context, state),
          WalletPersonalizeFailure() => _buildErrorPage(context),
          WalletPersonalizeDigidCancelled() => _buildDigidCancelledPage(context),
          WalletPersonalizeDigidFailure() => _buildDigidErrorPage(context),
          WalletPersonalizeNetworkError() => _buildNetworkError(context, state),
          WalletPersonalizeGenericError() => _buildGenericError(context, state),
          WalletPersonalizeSessionExpired() => _buildSessionExpired(context),
          WalletPersonalizeAddingCards() => _buildAddingCards(context, progress: state.stepperProgress),
          WalletPersonalizeRelyingPartyError() => _buildRelyingPartyError(context, state),
        };
        return FakePagingAnimatedSwitcher(animateBackwards: state.didGoBack, child: result);
      },
    );
  }

  /// Closes any dialogs opened on top of this [WalletPersonalizeScreen], ignored if none exist.
  void _closeOpenDialogs(BuildContext context) => Navigator.popUntil(context, (route) => route is! DialogRoute);

  Widget _buildCheckDataOfferingPage(BuildContext context, WalletPersonalizeCheckData state) {
    return WalletPersonalizeCheckDataOfferingPage(
      key: const Key('personalizePidPreviewPage'),
      onAcceptPressed: () => context.bloc.add(WalletPersonalizeOfferingAccepted(state.availableAttributes)),
      onRejectPressed: () => context.bloc.add(WalletPersonalizeOfferingRejected()),
      attributes: state.availableAttributes,
      progress: state.stepperProgress,
    );
  }

  Widget _buildLoading(BuildContext context, {VoidCallback? onCancel, FlowProgress? progress}) {
    return GenericLoadingPage(
      title: context.l10n.walletPersonalizeScreenLoadingTitle,
      description: context.l10n.walletPersonalizeScreenLoadingSubtitle,
      onCancel: onCancel,
      appBar: WalletAppBar(progress: progress),
    );
  }

  Widget _buildAddingCards(BuildContext context, {FlowProgress? progress}) {
    return GenericLoadingPage(
      title: context.l10n.walletPersonalizeScreenLoadingTitle,
      description: context.l10n.walletPersonalizeScreenAddingCardsSubtitle,
      appBar: WalletAppBar(progress: progress),
    );
  }

  Widget _buildAuthenticatingWithDigid(
    BuildContext context, {
    FlowProgress? progress,
    required DigiDAuthStage stage,
  }) {
    final title = switch (stage) {
      DigiDAuthStage.fetchingAuthUrl => context.l10n.walletPersonalizeScreenLoadingDigiDUrlTitle,
      DigiDAuthStage.awaitingUserAction => context.l10n.walletPersonalizeScreenAwaitingUserAuthTitle,
      DigiDAuthStage.processingResult => context.l10n.walletPersonalizeScreenProcessingDigiDResultTitle,
    };
    final description = switch (stage) {
      DigiDAuthStage.fetchingAuthUrl => context.l10n.walletPersonalizeScreenLoadingDigiDUrlDescription,
      DigiDAuthStage.awaitingUserAction => context.l10n.walletPersonalizeScreenAwaitingUserAuthDescription,
      DigiDAuthStage.processingResult => context.l10n.walletPersonalizeScreenProcessingDigiDResultDescription,
    };
    return GenericLoadingPage(
      key: const Key('personalizeAuthenticatingWithDigidPage'),
      title: title,
      description: description,
      cancelCta: context.l10n.walletPersonalizeScreenDigidLoadingStopCta,
      appBar: WalletAppBar(progress: progress),
      contextImage: Image.asset(WalletAssets.logo_wallet, height: 64, width: 64),
      onCancel: () async {
        final bloc = context.bloc;
        final cancelled = await _showStopDigidLoginDialog(context);
        if (cancelled) {
          bloc.add(
            WalletPersonalizeLoginWithDigidFailed(
              cancelledByUser: true,
              error: GenericError(cancelled, sourceError: Exception('Login cancelled')),
            ),
          );
        }
      },
      loadingIndicator: stage == DigiDAuthStage.awaitingUserAction ? const SizedBox.shrink() : const LoadingIndicator(),
    );
  }

  Future<bool> _showStopDigidLoginDialog(BuildContext context) async {
    /// This check helps avoid a race condition where the dialog is opened after the state change, meaning it would
    /// not be closed by [_closeOpenDialogs] as intended.
    final isAuthenticating = context.bloc.state is WalletPersonalizeAuthenticating;
    final isConnectingToDigid = context.bloc.state is WalletPersonalizeConnectDigid;
    final shouldShowDialog = isAuthenticating || isConnectingToDigid;
    if (!shouldShowDialog) return false;

    return StopDigidLoginDialog.show(context);
  }

  Widget _buildWalletIntroPage(BuildContext context, WalletPersonalizeInitial state) {
    return WalletPersonalizeIntroPage(
      key: const Key('personalizeInformPage'),
      onDigidLoginPressed: () => context.bloc.add(WalletPersonalizeLoginWithDigidClicked()),
      onDigidWebsitePressed: _launchDigidWebsite,
      progress: state.stepperProgress,
    );
  }

  Future<void> _loginWithDigid(BuildContext context, String authUrl) async {
    final bloc = context.bloc;
    if (Environment.mockRepositories) {
      await _performMockDigidLogin(context);
    } else {
      try {
        await launchUrlString(authUrl, mode: LaunchMode.externalApplication);
      } catch (ex) {
        Fimber.e('Failed to open auth url: $authUrl', ex: ex);
        bloc.add(WalletPersonalizeLoginWithDigidFailed(error: GenericError(ex.toString(), sourceError: ex)));
      }
    }
  }

  /// Initiate the mock digid login and and trigger [PidIssuanceNavigationRequest] on success
  Future<void> _performMockDigidLogin(BuildContext context) async {
    assert(Environment.mockRepositories, 'This flow is only intended for mock builds');

    final success = await MockDigidScreen.mockLogin(context);
    if (success && context.mounted) {
      await context.read<NavigationService>().handleNavigationRequest(PidIssuanceNavigationRequest('issue_pid'));
    } else if (context.mounted) {
      final error = GenericError('Mock login failed', sourceError: Exception('Mock exception'));
      context.bloc.add(WalletPersonalizeLoginWithDigidFailed(error: error));
    }
  }

  Widget _buildSuccessPage(BuildContext context, WalletPersonalizeSuccess state) {
    return Scaffold(
      appBar: WalletAppBar(
        progress: state.stepperProgress,
        title: _buildFadeInTitle(context.l10n.walletPersonalizeSuccessPageTitle),
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
      appBar: WalletAppBar(
        progress: const FlowProgress(currentStep: 0, totalSteps: kSetupSteps),
        title: _buildFadeInTitle(context.l10n.walletPersonalizeScreenErrorTitle),
      ),
      body: TerminalPage(
        illustration: const PageIllustration(asset: WalletAssets.svg_error_general),
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
        progress: FlowProgress(currentStep: 0, totalSteps: kSetupSteps),
      ),
      body: TerminalPage(
        title: context.l10n.walletPersonalizeDigidCancelledPageTitle,
        illustration: const PageIllustration(asset: WalletAssets.svg_stopped),
        description: context.l10n.walletPersonalizeDigidCancelledPageDescription,
        onPrimaryPressed: () => context.bloc.add(WalletPersonalizeLoginWithDigidClicked()),
        primaryButtonCta: context.l10n.walletPersonalizeDigidErrorPageLoginWithDigidCta,
        primaryButtonIcon: const SvgOrImage(asset: WalletAssets.logo_digid),
        onSecondaryButtonPressed: _launchDigidWebsite,
        secondaryButtonCta: context.l10n.walletPersonalizeDigidErrorPageDigidWebsiteCta,
        secondaryButtonIcon: const Icon(Icons.arrow_outward_rounded),
      ),
    );
  }

  Widget _buildDigidErrorPage(BuildContext context) {
    return Scaffold(
      appBar: const WalletAppBar(
        progress: FlowProgress(currentStep: 0, totalSteps: kSetupSteps),
      ),
      body: TerminalPage(
        title: context.l10n.walletPersonalizeDigidErrorPageTitle,
        illustration: const PageIllustration(asset: WalletAssets.svg_error_general),
        description: context.l10n.walletPersonalizeDigidErrorPageDescription,
        onPrimaryPressed: () => context.bloc.add(WalletPersonalizeLoginWithDigidClicked()),
        primaryButtonCta: context.l10n.walletPersonalizeDigidErrorPageLoginWithDigidCta,
        primaryButtonIcon: const SvgOrImage(asset: WalletAssets.logo_digid),
        onSecondaryButtonPressed: _launchDigidWebsite,
        secondaryButtonCta: context.l10n.walletPersonalizeDigidErrorPageDigidWebsiteCta,
        secondaryButtonIcon: const Icon(Icons.arrow_outward_rounded),
      ),
    );
  }

  Future<void> _showExitSheet(BuildContext context) async {
    assert(Platform.isAndroid, 'This should only be reachable through the back button on Android');
    final confirmed = await ConfirmActionSheet.show(
      context,
      title: context.l10n.walletPersonalizeScreenExitSheetTitle,
      description: context.l10n.walletPersonalizeScreenExitSheetDescription,
      confirmButton: ConfirmSheetButtonStyle(
        cta: context.l10n.walletPersonalizeScreenExitSheetConfirmCta,
        color: context.colorScheme.error,
      ),
      cancelButton: ConfirmSheetButtonStyle(cta: context.l10n.walletPersonalizeScreenExitSheetCancelCta),
    );
    if (confirmed && context.mounted) {
      if (Platform.isAndroid) {
        await SystemNavigator.pop();
      } else {
        // If we somehow reach this state on non-android platforms, kill the app the hard way
        exit(0);
      }
    }
  }

  Widget _buildConfirmPinPage(BuildContext context, WalletPersonalizeConfirmPin state) {
    return Scaffold(
      appBar: WalletAppBar(
        progress: state.stepperProgress,
        leading: _buildBackButton(context),
      ),
      body: WalletPersonalizeConfirmPinPage(
        onPidAccepted: (_) => context.bloc.add(WalletPersonalizePinConfirmed()),
        onAcceptPidFailed: (context, state) => context.bloc.add(WalletPersonalizeAcceptPidFailed(error: state.error)),
      ),
    );
  }

  Widget _buildNetworkError(BuildContext context, WalletPersonalizeNetworkError state) {
    if (state.hasInternet) {
      return Scaffold(
        appBar: WalletAppBar(
          progress: state.stepperProgress,
          title: _buildFadeInTitle(context.l10n.errorScreenServerHeadline),
        ),
        body: ErrorPage.network(
          context,
          style: ErrorCtaStyle.retry,
          onPrimaryActionPressed: () => context.bloc.add(WalletPersonalizeRetryPressed()),
        ),
      );
    } else {
      return Scaffold(
        appBar: WalletAppBar(
          progress: state.stepperProgress,
          title: _buildFadeInTitle(context.l10n.errorScreenNoInternetHeadline),
        ),
        body: ErrorPage.noInternet(
          context,
          onPrimaryActionPressed: () => context.bloc.add(WalletPersonalizeRetryPressed()),
          style: ErrorCtaStyle.retry,
        ),
      );
    }
  }

  Widget _buildGenericError(BuildContext context, WalletPersonalizeGenericError state) {
    return Scaffold(
      appBar: WalletAppBar(
        progress: state.stepperProgress,
        title: _buildFadeInTitle(context.l10n.errorScreenGenericHeadline),
      ),
      body: ErrorPage.generic(
        context,
        style: ErrorCtaStyle.retry,
        onPrimaryActionPressed: () => context.bloc.add(WalletPersonalizeRetryPressed()),
      ),
    );
  }

  Widget _buildRelyingPartyError(BuildContext context, WalletPersonalizeRelyingPartyError state) {
    return Scaffold(
      appBar: WalletAppBar(
        progress: state.stepperProgress,
        title: _buildFadeInTitle(context.l10n.genericRelyingPartyErrorTitle),
      ),
      body: ErrorPage.relyingParty(
        context,
        organizationName: state.organizationName?.l10nValue(context),
        onPrimaryActionPressed: () => context.bloc.add(WalletPersonalizeRetryPressed()),
      ),
    );
  }

  Widget _buildSessionExpired(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        progress: const FlowProgress(currentStep: 0, totalSteps: kSetupSteps),
        title: _buildFadeInTitle(context.l10n.errorScreenSessionExpiredHeadline),
      ),
      body: ErrorPage.sessionExpired(
        context,
        style: ErrorCtaStyle.retry,
        onPrimaryActionPressed: () => context.bloc.add(WalletPersonalizeRetryPressed()),
      ),
    );
  }

  Widget _buildFadeInTitle(String title) {
    return FadeInAtOffset(
      appearOffset: 50,
      visibleOffset: 100,
      child: TitleText(title),
    );
  }

  void _launchDigidWebsite() => launchUrlStringCatching(kDigidWebsiteUrl, mode: LaunchMode.externalApplication);
}

extension _WalletPersonalizeScreenExtension on BuildContext {
  WalletPersonalizeBloc get bloc => read<WalletPersonalizeBloc>();
}

enum DigiDAuthStage { fetchingAuthUrl, awaitingUserAction, processingResult }
