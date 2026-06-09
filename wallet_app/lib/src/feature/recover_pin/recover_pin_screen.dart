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
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/list_extension.dart';
import '../../util/extension/navigator_state_extension.dart';
import '../../util/extension/object_extension.dart';
import '../../util/launch_util.dart';
import '../../wallet_assets.dart';
import '../../wallet_constants.dart';
import '../../wallet_core/error/core_error.dart';
import '../common/dialog/stop_digid_login_dialog.dart';
import '../common/mixin/lock_state_mixin.dart';
import '../common/page/generic_loading_page.dart';
import '../common/page/terminal_page.dart';
import '../common/widget/button/icon/back_icon_button.dart';
import '../common/widget/button/icon/close_icon_button.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/button/primary_button.dart';
import '../common/widget/button/tertiary_button.dart';
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

class RecoverPinScreen extends StatefulWidget {
  const RecoverPinScreen({super.key});

  @override
  State<RecoverPinScreen> createState() => _RecoverPinScreenState();
}

class _RecoverPinScreenState extends State<RecoverPinScreen> with LockStateMixin<RecoverPinScreen> {
  @override
  FutureOr<void> onLock() => Navigator.of(context).resetToDashboard();

  @override
  FutureOr<void> onUnlock() {}

  @override
  Widget build(BuildContext context) {
    final state = context.watch<RecoverPinBloc>().state;
    final title = _resolvePageTitle(context, state);
    return Scaffold(
      appBar: WalletAppBar(
        automaticallyImplyLeading: false,
        title: title == null ? const SizedBox.shrink() : TitleText(title),
        leading: BackIconButton(
          onPressed: () => _handleBackPress(state, context),
        ).takeIf((_) => _canPop(state)),
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
          if (!didPop) _handleBackPress(state, context);
        },
        child: SafeArea(
          child: _buildPage(),
        ),
      ),
    );
  }

  void _handleBackPress(RecoverPinState state, BuildContext context) {
    if (state.canGoBack) {
      context.addBackPressedEvent();
    } else if (Navigator.canPop(context)) {
      Navigator.pop(context);
    } else {
      Navigator.of(context).resetToDashboard();
    }
  }

  /// Returns the title to be shown at the top of the page, or null if none should be visible for the provided [state].
  String? _resolvePageTitle(BuildContext context, RecoverPinState state) {
    switch (state) {
      case RecoverPinInitial():
        return context.l10n.recoverPinIntroPageTitle;
      case RecoverPinDigidMismatch():
        return context.l10n.recoverPinDigidMismatchPageTitle;
      case RecoverPinStopped():
        return context.l10n.recoverPinStoppedPageTitle;
      case RecoverPinSuccess():
        return context.l10n.recoverPinSuccessPageTitle;
      case RecoverPinDigidLoginCancelled():
        return context.l10n.recoverPinLoginCancelledPageTitle;
      case RecoverPinDigidFailure():
        return context.l10n.recoverPinGenericErrorTitle;
      case RecoverPinError(:final error):
        return ErrorPage.titleFromError(context, error);
      default:
        return null;
    }
  }

  /// Determines if the "Back" button should be displayed in the AppBar,
  /// and if the system back gesture should be available..
  ///
  /// Returns:
  ///   True if the "Back" button should be visible, false otherwise.
  bool _canPop(RecoverPinState state) {
    if (state.canGoBack) return true;
    return switch (state) {
      RecoverPinInitial() => true,
      RecoverPinLoadingDigidUrl() => false,
      RecoverPinAwaitingDigidAuthentication() => false,
      RecoverPinVerifyingDigidAuthentication() => false,
      RecoverPinDigidMismatch() => true,
      RecoverPinStopped() => false,
      RecoverPinChooseNewPin() => false,
      RecoverPinConfirmNewPin() => false,
      RecoverPinUpdatingPin() => false,
      RecoverPinSuccess() => false,
      RecoverPinSelectPinFailed() => true,
      RecoverPinConfirmPinFailed() => true,
      RecoverPinDigidFailure() => true,
      RecoverPinDigidLoginCancelled() => true,
      RecoverPinError() => true,
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
      RecoverPinError() => false,
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
      builder: (context, state) {
        final Widget page = switch (state) {
          RecoverPinInitial() => _buildInitial(context, state),
          RecoverPinLoadingDigidUrl() => _buildLoadingDigidUrl(context),
          RecoverPinAwaitingDigidAuthentication() => _buildAwaitingDigidAuthentication(context),
          RecoverPinVerifyingDigidAuthentication() => _buildVerifyingDigidAuthentication(context),
          RecoverPinDigidMismatch() => _buildDigidMismatch(context, state),
          RecoverPinStopped() => _buildStopped(context, state),
          RecoverPinChooseNewPin() => _buildChooseNewPin(context, state),
          RecoverPinConfirmNewPin() => _buildConfirmNewPin(context, state),
          RecoverPinUpdatingPin() => _buildUpdatingPin(context),
          RecoverPinSuccess() => _buildSuccess(context, state),
          RecoverPinSelectPinFailed() => const CenteredLoadingIndicator(),
          RecoverPinConfirmPinFailed() => const CenteredLoadingIndicator(),
          RecoverPinDigidFailure() => _buildDigidFailure(context, state),
          RecoverPinDigidLoginCancelled() => _buildLoginCancelled(context, state),
          RecoverPinError(:final error) => _buildError(context, error),
        };
        return FakePagingAnimatedSwitcher(
          animateBackwards: state.didGoBack,
          child: page,
        );
      },
    );
  }

  Widget _buildInitial(BuildContext context, RecoverPinInitial state) {
    return TerminalPage(
      title: _resolvePageTitle(context, state) ?? '',
      description: context.l10n.recoverPinIntroPageDescription,
      illustration: const PageIllustration(asset: WalletAssets.svg_digid),
      primaryButton: PrimaryButton(
        text: Text(context.l10n.recoverPinIntroPageOpenDigidCta),
        icon: const SvgOrImage(asset: WalletAssets.logo_digid),
        onPressed: () => context.bloc.add(const RecoverPinLoginWithDigidClicked()),
        key: const Key('primaryButtonCta'),
      ),
      secondaryButton: TertiaryButton(
        text: Text(context.l10n.recoverPinOpenDigidWebsiteCta),
        icon: const Icon(Icons.north_east_outlined),
        onPressed: _launchDigidWebsite,
        key: const Key('secondaryButtonCta'),
      ),
    );
  }

  Widget _buildLoadingDigidUrl(BuildContext context) {
    return GenericLoadingPage(
      title: context.l10n.recoverPinGenericLoadingTitle,
      description: context.l10n.recoverPinLoadingDigidUrlDescription,
    );
  }

  Widget _buildAwaitingDigidAuthentication(BuildContext context) {
    return GenericLoadingPage(
      title: context.l10n.recoverPinContinueWithDigiDTitle,
      description: context.l10n.recoverPinContinueWithDigiDDescription,
      cancelCta: context.l10n.generalStop,
      loadingIndicator: const SizedBox.shrink(),
      onCancel: () => _stopRecoverPin(context),
    );
  }

  Widget _buildVerifyingDigidAuthentication(BuildContext context) {
    return GenericLoadingPage(
      contextImage: const SvgOrImage(asset: WalletAssets.logo_wallet, height: 64, width: 64),
      title: context.l10n.recoverPinGenericLoadingTitle,
      description: context.l10n.recoverPinVerifyingAuthenticationDescription,
      cancelCta: context.l10n.generalStop,
      onCancel: () => _stopRecoverPin(context),
    );
  }

  Widget _buildChooseNewPin(BuildContext context, RecoverPinChooseNewPin state) {
    return PinSetupPage(
      title: context.l10n.recoverPinChooseNewPinPageTitle,
      enteredDigits: state.enteredDigits,
      onBackspacePressed: () => context.bloc.add(RecoverPinBackspacePressed()),
      onBackspaceLongPressed: () => context.bloc.add(RecoverPinClearPressed()),
      onKeyPressed: (digit) => context.bloc.add(RecoverPinDigitPressed(digit)),
      onStopPressed: () => _stopRecoverPin(context),
    );
  }

  Widget _buildConfirmNewPin(BuildContext context, RecoverPinConfirmNewPin state) {
    return PinSetupPage(
      title: context.l10n.recoverPinConfirmNewPinPageTitle,
      enteredDigits: state.enteredDigits,
      onBackspacePressed: () => context.bloc.add(RecoverPinBackspacePressed()),
      onBackspaceLongPressed: () => context.bloc.add(RecoverPinClearPressed()),
      onKeyPressed: (digit) => context.bloc.add(RecoverPinDigitPressed(digit)),
      onStopPressed: () => _stopRecoverPin(context),
    );
  }

  Widget _buildUpdatingPin(BuildContext context) {
    return GenericLoadingPage(
      contextImage: const SvgOrImage(asset: WalletAssets.logo_wallet, height: 64, width: 64),
      title: context.l10n.recoverPinGenericLoadingTitle,
      description: context.l10n.recoverPinUpdatingPinPageDescription,
    );
  }

  Widget _buildError(BuildContext context, ApplicationError error) {
    return ErrorPage.fromError(
      context,
      error,
      onPrimaryActionPressed: () => context.bloc.add(const RecoverPinRetryPressed()),
      style: ErrorCtaStyle.retry,
    );
  }

  Widget _buildLoginCancelled(BuildContext context, RecoverPinDigidLoginCancelled state) {
    return TerminalPage(
      title: _resolvePageTitle(context, state) ?? '',
      description: context.l10n.recoverPinLoginCancelledPageDescription,
      illustration: const PageIllustration(asset: WalletAssets.svg_stopped),
      primaryButton: PrimaryButton(
        text: Text(context.l10n.recoverPinLoginCancelledPageRetryCta),
        icon: const SvgOrImage(asset: WalletAssets.logo_digid),
        onPressed: () => context.bloc.add(const RecoverPinLoginWithDigidClicked()),
        key: const Key('primaryButtonCta'),
      ),
      secondaryButton: TertiaryButton(
        text: Text(context.l10n.recoverPinOpenDigidWebsiteCta),
        icon: const Icon(Icons.north_east_outlined),
        onPressed: _launchDigidWebsite,
        key: const Key('secondaryButtonCta'),
      ),
    );
  }

  Widget _buildDigidFailure(BuildContext context, RecoverPinDigidFailure state) {
    return TerminalPage(
      title: _resolvePageTitle(context, state) ?? '',
      description: context.l10n.recoverPinFailurePageDescription,
      illustration: const PageIllustration(asset: WalletAssets.svg_stopped),
      primaryButton: PrimaryButton(
        text: Text(context.l10n.recoverPinFailurePageRetryCta),
        icon: const SvgOrImage(asset: WalletAssets.logo_digid),
        onPressed: () => context.bloc.add(const RecoverPinLoginWithDigidClicked()),
        key: const Key('primaryButtonCta'),
      ),
      secondaryButton: TertiaryButton(
        text: Text(context.l10n.recoverPinOpenDigidWebsiteCta),
        icon: const Icon(Icons.north_east_outlined),
        onPressed: _launchDigidWebsite,
        key: const Key('secondaryButtonCta'),
      ),
    );
  }

  Widget _buildSuccess(BuildContext context, RecoverPinSuccess state) {
    return TerminalPage(
      title: _resolvePageTitle(context, state) ?? '',
      description: context.l10n.recoverPinSuccessPageDescription,
      illustration: const PageIllustration(
        asset: WalletAssets.svg_pin_set,
      ),
      primaryButton: PrimaryButton(
        text: Text(context.l10n.recoverPinSuccessPageToOverviewCta),
        onPressed: () => Navigator.of(context).resetToDashboard(),
        key: const Key('primaryButtonCta'),
      ),
      secondaryButton: TertiaryButton(
        text: Text(context.l10n.recoverPinSuccessPageToHistoryCta),
        icon: const Icon(Icons.north_east_outlined),
        onPressed: () => Navigator.of(context)
          ..resetToDashboard()
          ..pushNamed(WalletRoutes.walletHistoryRoute),
        key: const Key('secondaryButtonCta'),
      ).takeIf((_) => false), // Reintroduce with PVW-5058 (also see PVW-5141)
    );
  }

  Widget _buildStopped(BuildContext context, RecoverPinStopped state) {
    return TerminalPage(
      title: _resolvePageTitle(context, state) ?? '',
      description: context.l10n.recoverPinStoppedPageDescription,
      illustration: const PageIllustration(
        asset: WalletAssets.svg_stopped,
      ),
      primaryButton: PrimaryButton(
        text: Text(context.l10n.generalClose),
        icon: const Icon(Icons.close_outlined),
        onPressed: () => Navigator.of(context).resetToDashboard(),
        key: const Key('primaryButtonCta'),
      ),
    );
  }

  Widget _buildDigidMismatch(BuildContext context, RecoverPinDigidMismatch state) {
    return TerminalPage(
      title: _resolvePageTitle(context, state) ?? '',
      description: context.l10n.recoverPinDigidMismatchPageDescription,
      illustration: const PageIllustration(
        asset: WalletAssets.svg_error_general,
      ),
      primaryButton: PrimaryButton(
        text: Text(context.l10n.recoverPinDigidMismatchPageRetryCta),
        icon: const SvgOrImage(asset: WalletAssets.logo_digid),
        onPressed: () => context.bloc.add(const RecoverPinRetryPressed()),
        key: const Key('primaryButtonCta'),
      ),
      secondaryButton: TertiaryButton(
        text: Text(context.l10n.recoverPinOpenDigidWebsiteCta),
        icon: const Icon(Icons.north_east_outlined),
        onPressed: _launchDigidWebsite,
        key: const Key('secondaryButtonCta'),
      ),
    );
  }

  /// Stop the recover PIN flow, this methods checks the current state to make
  /// sure the correct stop action (dialog/sheet/pop) will be executed.
  Future<void> _stopRecoverPin(BuildContext context) async {
    final state = context.bloc.state;
    switch (state) {
      case ErrorState():
        Navigator.pop(context);
      case RecoverPinSuccess():
        unawaited(Navigator.of(context).resetToDashboard());
      case RecoverPinAwaitingDigidAuthentication():
        // This is a special case, for which we show the stop dialog
        unawaited(_showStopDigidLoginDialog(context));
      default:
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
        await launchUrl(Uri.parse(authUrl), mode: LaunchMode.externalApplication);
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
      await context.read<NavigationService>().handleNavigationRequest(NavigationRequest.pinRecovery('mock'));
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
