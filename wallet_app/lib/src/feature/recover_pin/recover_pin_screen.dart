import 'dart:async';

import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:url_launcher/url_launcher.dart';

import '../../../environment.dart';
import '../../data/service/navigation_service.dart';
import '../../domain/model/bloc/error_state.dart';
import '../../domain/model/navigation/navigation_request.dart';
import '../../domain/model/result/application_error.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/cast_util.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/list_extension.dart';
import '../../util/extension/navigator_state_extension.dart';
import '../../util/extension/object_extension.dart';
import '../../util/launch_util.dart';
import '../../wallet_assets.dart';
import '../../wallet_constants.dart';
import '../../wallet_core/error/core_error.dart';
import '../common/dialog/stop_digid_login_dialog.dart';
import '../common/page/generic_loading_page.dart';
import '../common/page/terminal_page.dart';
import '../common/widget/button/icon/back_icon_button.dart';
import '../common/widget/button/icon/close_icon_button.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/fake_paging_animated_switcher.dart';
import '../common/widget/page_illustration.dart';
import '../common/widget/svg_or_image.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../error/error_page.dart';
import '../mock_digid/mock_digid_screen.dart';
import '../pin/pin_setup_page.dart';
import '../pin_dialog/pin_confirmation_error_dialog.dart';
import '../pin_dialog/pin_validation_error_dialog.dart';
import 'bloc/recover_pin_bloc.dart';
import 'recover_pin_stop_sheet.dart';

class RecoverPinScreen extends StatelessWidget {
  const RecoverPinScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final state = context.watch<RecoverPinBloc>().state;
    return Scaffold(
      appBar: WalletAppBar(
        automaticallyImplyLeading: false,
        title: _buildTitle(context),
        leading: BackIconButton(
          onPressed: () => state.canGoBack ? context.addBackPressedEvent() : Navigator.pop(context),
        ).takeIf((_) => state.canGoBack || _canPop(state)),
        actions: [
          const HelpIconButton(),
          CloseIconButton(onPressed: () => _stopRecoverPin(context)).takeIf(
            (_) => _canStop(state),
          ),
        ].nonNullsList,
        progress: state.stepperProgress,
      ),
      body: PopScope(
        canPop: _canPop(state),
        onPopInvokedWithResult: (didPop, result) {
          if (!didPop) context.addBackPressedEvent();
        },
        child: SafeArea(
          child: _buildPage(),
        ),
      ),
    );
  }

  Widget _buildTitle(BuildContext context) {
    final state = context.watch<RecoverPinBloc>().state;
    switch (state) {
      case RecoverPinInitial():
        return TitleText(context.l10n.recoverPinIntroPageTitle);
      case RecoverPinDigidMismatch():
        return TitleText(context.l10n.recoverPinDigidMismatchPageTitle);
      case RecoverPinStopped():
        return TitleText(context.l10n.recoverPinStoppedPageTitle);
      case RecoverPinSuccess():
        return TitleText(context.l10n.recoverPinSuccessPageTitle);
      case RecoverPinDigidLoginCancelled():
        return TitleText(context.l10n.recoverPinLoginCancelledPageTitle);
      case RecoverPinDigidFailure():
      case RecoverPinNetworkError():
      case RecoverPinSessionExpired():
      case RecoverPinGenericError():
        return TitleText(context.l10n.recoverPinGenericErrorTitle);
      case RecoverPinSelectPinFailed():
      case RecoverPinConfirmPinFailed():
      case RecoverPinChooseNewPin():
      case RecoverPinConfirmNewPin():
      case RecoverPinUpdatingPin():
      case RecoverPinLoadingDigidUrl():
      case RecoverPinAwaitingDigidAuthentication():
      case RecoverPinVerifyingDigidAuthentication():
        return const SizedBox.shrink();
    }
  }

  /// Determines if the "Back" button should be displayed in the AppBar,
  /// and if the system back gesture should be available..
  ///
  /// Returns:
  ///   True if the "Back" button should be visible, false otherwise.
  bool _canPop(RecoverPinState state) {
    return switch (state) {
      RecoverPinInitial() => true,
      RecoverPinLoadingDigidUrl() => false,
      RecoverPinAwaitingDigidAuthentication() => false,
      RecoverPinVerifyingDigidAuthentication() => false,
      RecoverPinDigidMismatch() => true,
      RecoverPinStopped() => true,
      RecoverPinChooseNewPin() => false,
      RecoverPinConfirmNewPin() => false,
      RecoverPinUpdatingPin() => false,
      RecoverPinSuccess() => true,
      RecoverPinSelectPinFailed() => true,
      RecoverPinConfirmPinFailed() => true,
      RecoverPinDigidFailure() => true,
      RecoverPinDigidLoginCancelled() => true,
      RecoverPinNetworkError() => true,
      RecoverPinGenericError() => true,
      RecoverPinSessionExpired() => true,
    };
  }

  /// Determines if the "Stop" button should be displayed in the AppBar.
  ///
  /// Returns:
  ///   True if the "Stop" button should be visible, false otherwise.
  bool _canStop(RecoverPinState state) {
    return switch (state) {
      RecoverPinInitial() => false,
      RecoverPinLoadingDigidUrl() => false,
      RecoverPinAwaitingDigidAuthentication() => true,
      RecoverPinVerifyingDigidAuthentication() => false,
      RecoverPinDigidMismatch() => false,
      RecoverPinStopped() => false,
      RecoverPinChooseNewPin() => true,
      RecoverPinConfirmNewPin() => true,
      RecoverPinUpdatingPin() => false,
      RecoverPinSuccess() => false,
      RecoverPinSelectPinFailed() => false,
      RecoverPinConfirmPinFailed() => false,
      RecoverPinDigidFailure() => false,
      RecoverPinDigidLoginCancelled() => false,
      RecoverPinNetworkError() => false,
      RecoverPinGenericError() => false,
      RecoverPinSessionExpired() => false,
    };
  }

  Widget _buildPage() {
    return BlocConsumer<RecoverPinBloc, RecoverPinState>(
      listener: (c, state) {
        StopDigidLoginDialog.closeOpenDialog(c); // Close StopDigiD dialog in case of state changes
        if (state is RecoverPinSelectPinFailed) PinValidationErrorDialog.show(c, state.reason);
        if (state is RecoverPinConfirmPinFailed) PinConfirmationErrorDialog.show(c, retryAllowed: state.canRetry);
        if (state is RecoverPinAwaitingDigidAuthentication) _loginWithDigid(c, state.authUrl);
      },
      buildWhen: (prev, current) {
        // Check for states that ONLY trigger a dialog, but do not require a UI rebuild.
        if (current is RecoverPinSelectPinFailed) return false;
        if (current is RecoverPinConfirmPinFailed) return false;
        return true;
      },
      builder: (c, state) {
        final pageTitle = tryCast<TitleText>(_buildTitle(c))?.data ?? '';
        Widget page;
        switch (state) {
          case RecoverPinInitial():
            page = TerminalPage(
              title: pageTitle,
              description: c.l10n.recoverPinIntroPageDescription,
              primaryButtonCta: c.l10n.recoverPinIntroPageOpenDigidCta,
              primaryButtonIcon: const SvgOrImage(asset: WalletAssets.logo_digid),
              illustration: const PageIllustration(asset: WalletAssets.svg_digid),
              onPrimaryPressed: () => c.bloc.add(const RecoverPinLoginWithDigidClicked()),
              onSecondaryButtonPressed: _launchDigidWebsite,
              secondaryButtonCta: c.l10n.recoverPinOpenDigidWebsiteCta,
              secondaryButtonIcon: const Icon(Icons.north_east_outlined),
            );
          case RecoverPinLoadingDigidUrl():
            page = GenericLoadingPage(
              title: c.l10n.recoverPinGenericLoadingTitle,
              description: c.l10n.recoverPinLoadingDigidUrlDescription,
            );
          case RecoverPinAwaitingDigidAuthentication():
            page = GenericLoadingPage(
              contextImage: const SvgOrImage(asset: WalletAssets.logo_wallet, height: 64, width: 64),
              title: c.l10n.recoverPinGenericLoadingTitle,
              description: c.l10n.recoverPinAwaitingDigidAuthenticationDescription,
              cancelCta: c.l10n.generalStop,
              onCancel: () => _stopRecoverPin(c),
            );
          case RecoverPinVerifyingDigidAuthentication():
            page = GenericLoadingPage(
              contextImage: const SvgOrImage(asset: WalletAssets.logo_wallet, height: 64, width: 64),
              title: c.l10n.recoverPinGenericLoadingTitle,
              description: c.l10n.recoverPinVerifyingAuthenticationDescription,
              cancelCta: c.l10n.generalStop,
              onCancel: () => _stopRecoverPin(c),
            );
          case RecoverPinDigidMismatch():
            page = TerminalPage(
              title: pageTitle,
              description: c.l10n.recoverPinDigidMismatchPageDescription,
              illustration: const PageIllustration(asset: WalletAssets.svg_error_general),
              primaryButtonCta: c.l10n.recoverPinDigidMismatchPageRetryCta,
              onPrimaryPressed: () => c.bloc.add(const RecoverPinRetryPressed()),
              primaryButtonIcon: const SvgOrImage(asset: WalletAssets.logo_digid),
              secondaryButtonCta: c.l10n.recoverPinOpenDigidWebsiteCta,
              onSecondaryButtonPressed: _launchDigidWebsite,
              secondaryButtonIcon: const Icon(Icons.north_east_outlined),
            );
          case RecoverPinStopped():
            page = TerminalPage(
              title: pageTitle,
              description: c.l10n.recoverPinStoppedPageDescription,
              primaryButtonCta: c.l10n.generalClose,
              illustration: const PageIllustration(asset: WalletAssets.svg_stopped),
              onPrimaryPressed: () => Navigator.pop(c),
              primaryButtonIcon: const Icon(Icons.close_outlined),
            );
          case RecoverPinChooseNewPin():
            page = PinSetupPage(
              title: c.l10n.recoverPinChooseNewPinPageTitle,
              enteredDigits: state.enteredDigits,
              onBackspacePressed: () => c.bloc.add(RecoverPinBackspacePressed()),
              onBackspaceLongPressed: () => c.bloc.add(RecoverPinClearPressed()),
              onKeyPressed: (digit) => c.bloc.add(RecoverPinDigitPressed(digit)),
              onStopPressed: () => _stopRecoverPin(c),
            );
          case RecoverPinConfirmNewPin():
            page = PinSetupPage(
              title: c.l10n.recoverPinConfirmNewPinPageTitle,
              enteredDigits: state.enteredDigits,
              onBackspacePressed: () => c.bloc.add(RecoverPinBackspacePressed()),
              onBackspaceLongPressed: () => c.bloc.add(RecoverPinClearPressed()),
              onKeyPressed: (digit) => c.bloc.add(RecoverPinDigitPressed(digit)),
              onStopPressed: () => _stopRecoverPin(c),
            );
          case RecoverPinUpdatingPin():
            page = GenericLoadingPage(
              contextImage: const SvgOrImage(asset: WalletAssets.logo_wallet, height: 64, width: 64),
              title: c.l10n.recoverPinGenericLoadingTitle,
              description: c.l10n.recoverPinUpdatingPinPageDescription,
            );
          case RecoverPinSuccess():
            page = TerminalPage(
              title: pageTitle,
              description: c.l10n.recoverPinSuccessPageDescription,
              illustration: const PageIllustration(asset: WalletAssets.svg_pin_set),
              primaryButtonCta: c.l10n.recoverPinSuccessPageToOverviewCta,
              onPrimaryPressed: () => Navigator.of(c).resetToDashboard(),
              secondaryButtonCta: c.l10n.recoverPinSuccessPageToHistoryCta,
              onSecondaryButtonPressed: () {
                Navigator.of(c)
                  ..resetToDashboard()
                  ..pushNamed(WalletRoutes.walletHistoryRoute);
              },
            );
          case RecoverPinSelectPinFailed():
            page = const CenteredLoadingIndicator();
          case RecoverPinConfirmPinFailed():
            page = const CenteredLoadingIndicator();
          case RecoverPinDigidFailure():
            page = TerminalPage(
              title: pageTitle,
              description: c.l10n.recoverPinFailurePageDescription,
              primaryButtonCta: c.l10n.recoverPinFailurePageRetryCta,
              illustration: const PageIllustration(asset: WalletAssets.svg_stopped),
              onPrimaryPressed: () => c.bloc.add(const RecoverPinLoginWithDigidClicked()),
              primaryButtonIcon: const SvgOrImage(asset: WalletAssets.logo_digid),
              secondaryButtonIcon: const Icon(Icons.north_east_outlined),
              secondaryButtonCta: c.l10n.recoverPinOpenDigidWebsiteCta,
              onSecondaryButtonPressed: _launchDigidWebsite,
            );
          case RecoverPinDigidLoginCancelled():
            page = TerminalPage(
              title: pageTitle,
              description: c.l10n.recoverPinLoginCancelledPageDescription,
              primaryButtonCta: c.l10n.recoverPinLoginCancelledPageRetryCta,
              illustration: const PageIllustration(asset: WalletAssets.svg_stopped),
              onPrimaryPressed: () => c.bloc.add(const RecoverPinLoginWithDigidClicked()),
              primaryButtonIcon: const SvgOrImage(asset: WalletAssets.logo_digid),
              secondaryButtonIcon: const Icon(Icons.north_east_outlined),
              secondaryButtonCta: c.l10n.recoverPinOpenDigidWebsiteCta,
              onSecondaryButtonPressed: _launchDigidWebsite,
            );
          case RecoverPinNetworkError():
            page = state.hasInternet
                ? ErrorPage.network(
                    c,
                    onPrimaryActionPressed: () => c.bloc.add(const RecoverPinRetryPressed()),
                    style: ErrorCtaStyle.retry,
                  )
                : ErrorPage.noInternet(
                    c,
                    onPrimaryActionPressed: () => c.bloc.add(const RecoverPinRetryPressed()),
                    style: ErrorCtaStyle.retry,
                  );
          case RecoverPinGenericError():
            page = ErrorPage.generic(
              c,
              onPrimaryActionPressed: () => c.bloc.add(const RecoverPinRetryPressed()),
              style: ErrorCtaStyle.retry,
            );
          case RecoverPinSessionExpired():
            page = ErrorPage.sessionExpired(
              c,
              onPrimaryActionPressed: () => c.bloc.add(const RecoverPinRetryPressed()),
              style: ErrorCtaStyle.retry,
            );
        }
        return FakePagingAnimatedSwitcher(
          animateBackwards: state.didGoBack,
          child: page,
        );
      },
    );
  }

  /// Stop the recover PIN flow, this methods checks the current state to make
  /// sure the correct stop action (dialog/sheet/pop) will be executed.
  Future<void> _stopRecoverPin(BuildContext context) async {
    final state = context.bloc.state;
    if (state is ErrorState) {
      Navigator.pop(context);
    } else if (state is RecoverPinAwaitingDigidAuthentication) {
      // This is a special case, for which we show the stop dialog
      unawaited(_showStopDigidLoginDialog(context));
    } else {
      // Confirm with user through the stop sheet
      final stoppedByUser = await RecoverPinStopSheet.show(context);
      if (stoppedByUser && context.mounted) context.bloc.add(const RecoverPinStopPressed());
    }
  }

  /// Show a stop login dialog, only shown when the app is in the [RecoverPinAwaitingDigidAuthentication] state.
  /// When user confirms the stop action the login with DigiD session is aborted.
  Future<void> _showStopDigidLoginDialog(BuildContext context) async {
    if (context.bloc.state is! RecoverPinAwaitingDigidAuthentication) return; // Sanity check to avoid race conditions.
    final cancelled = await StopDigidLoginDialog.show(context);
    if (cancelled && context.mounted) {
      context.bloc.add(
        const RecoverPinLoginWithDigidFailed(
          cancelledByUser: true,
          error: GenericError('cancelled_by_user', sourceError: 'user'),
        ),
      );
    }
  }

  /// Initiates the (external) DigiD login by launching the provided authUrl
  Future<void> _loginWithDigid(BuildContext context, String authUrl) async {
    if (Environment.mockRepositories) {
      await _performMockLogin(context);
    } else {
      try {
        await launchUrl(Uri.parse(authUrl), mode: LaunchMode.platformDefault);
      } catch (ex) {
        final error = GenericError('Failed to launch digid url', sourceError: ex);
        if (context.mounted) {
          context.bloc.add(RecoverPinLaunchDigidUrlFailed(error: error));
        } else {
          Fimber.e('Failed to notify bloc', ex: error);
        }
      }
    }
  }

  /// Initiate the mock digid login and and trigger [PinRecoveryNavigationRequest] on success
  Future<void> _performMockLogin(BuildContext context) async {
    assert(Environment.mockRepositories, 'Mock login is intended for mock builds only');
    final success = await MockDigidScreen.mockLogin(context);
    if (success && context.mounted) {
      await context.read<NavigationService>().handleNavigationRequest(PinRecoveryNavigationRequest('renew_pid'));
    } else if (context.mounted) {
      context.bloc.add(
        const RecoverPinLoginWithDigidFailed(
          error: RedirectUriError(redirectError: RedirectError.loginRequired, sourceError: 'mock'),
        ),
      );
    }
  }

  /// Launches the DigiD help page in the external browser
  void _launchDigidWebsite() => launchUrlStringCatching(kDigidWebsiteUrl, mode: LaunchMode.externalApplication);
}

extension _RecoverPinScreenExtension on BuildContext {
  RecoverPinBloc get bloc => read<RecoverPinBloc>();

  void addBackPressedEvent() => bloc.add(const RecoverPinBackPressed());
}
