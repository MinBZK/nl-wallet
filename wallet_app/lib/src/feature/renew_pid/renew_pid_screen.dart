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
import '../../util/launch_util.dart';
import '../../wallet_assets.dart';
import '../../wallet_constants.dart';
import '../../wallet_core/error/core_error.dart';
import '../common/dialog/stop_digid_login_dialog.dart';
import '../common/page/generic_loading_page.dart';
import '../common/widget/button/icon/back_icon_button.dart';
import '../common/widget/button/icon/close_icon_button.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/fade_in_at_offset.dart';
import '../common/widget/fake_paging_animated_switcher.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../error/error_page.dart';
import '../mock_digid/mock_digid_screen.dart';
import 'bloc/renew_pid_bloc.dart';
import 'page/renew_pid_check_details_page.dart';
import 'page/renew_pid_confirm_pin_page.dart';
import 'page/renew_pid_digid_mismatch_page.dart';
import 'page/renew_pid_initial_page.dart';
import 'page/renew_pid_login_cancelled_page.dart';
import 'page/renew_pid_stopped_page.dart';
import 'page/renew_pid_success_page.dart';
import 'renew_pid_stop_sheet.dart';

const _kOpeningDigidStateKey = Key('opening_digid');

class RenewPidScreen extends StatelessWidget {
  const RenewPidScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return ScrollOffsetProvider(
      child: BlocListener<RenewPidBloc, RenewPidState>(
        listener: (BuildContext context, RenewPidState state) {
          context.read<ScrollOffset>().reset(); // Reset provided scrollOffset between pages
          StopDigidLoginDialog.closeOpenDialog(context); // Close StopDigiD dialog in case of state changes
          if (state is RenewPidAwaitingDigidAuthentication) _loginWithDigid(context, state.authUrl);
        },
        child: Scaffold(
          appBar: _buildAppBar(context),
          body: _buildBody(context),
        ),
      ),
    );
  }

  PreferredSizeWidget _buildAppBar(BuildContext context) {
    final state = context.watch<RenewPidBloc>().state;
    return WalletAppBar(
      leading: state.canGoBack ? BackIconButton(onPressed: () => context.bloc.add(const RenewPidBackPressed())) : null,
      automaticallyImplyLeading: false,
      actions: [
        const HelpIconButton(),
        CloseIconButton(onPressed: () => _stopRenewPid(context)),
      ],
      title: _buildTitle(context, state),
      progress: state.stepperProgress,
    );
  }

  Widget _buildTitle(BuildContext context, RenewPidState state) {
    String title;
    switch (state) {
      case RenewPidInitial():
        title = context.l10n.renewPidIntroPageTitle;
      case RenewPidCheckData():
        title = context.l10n.renewPidCheckDetailsPageTitle;
      case RenewPidSuccess():
        title = context.l10n.renewPidSuccessPageTitle;
      case RenewPidConfirmPin():
      case RenewPidDigidFailure():
      case RenewPidNetworkError():
      case RenewPidGenericError():
      case RenewPidSessionExpired():
      case RenewPidLoadingDigidUrl():
      case RenewPidAwaitingDigidAuthentication():
      case RenewPidVerifyingDigidAuthentication():
      case RenewPidUpdatingCards():
        return const SizedBox.shrink();
      case RenewPidDigidLoginCancelled():
        title = context.l10n.renewPidDigidLoginCancelledTitle;
      case RenewPidDigidMismatch():
        title = context.l10n.renewPidDigidMismatchPageTitle;
      case RenewPidStopped():
        title = context.l10n.renewPidStoppedTitle;
    }
    return TitleText(title);
  }

  Widget _buildBody(BuildContext context) {
    return BlocBuilder<RenewPidBloc, RenewPidState>(
      builder: (context, state) {
        Widget result;
        switch (state) {
          case RenewPidInitial():
            result = RenewPidInitialPage(
              onPrimaryPressed: () => context.bloc.add(const RenewPidLoginWithDigidClicked()),
              onSecondaryButtonPressed: _launchDigidWebsite,
            );
          case RenewPidLoadingDigidUrl():
            result = GenericLoadingPage(
              key: _kOpeningDigidStateKey,
              title: context.l10n.renewPidLoadingDigidUrlTitle,
              description: context.l10n.renewPidLoadingDigidUrlDescription,
            );
          case RenewPidAwaitingDigidAuthentication():
            result = GenericLoadingPage(
              key: _kOpeningDigidStateKey,
              title: context.l10n.renewPidOpeningDigidTitle,
              description: context.l10n.renewPidOpeningDigidDescription,
              contextImage: Image.asset(WalletAssets.logo_wallet, height: 64, width: 64),
              onCancel: () => _stopRenewPid(context),
              cancelCta: context.l10n.renewPidOpeningDigidStopCta,
              loadingIndicator: const SizedBox.shrink(),
            );
          case RenewPidVerifyingDigidAuthentication():
            result = GenericLoadingPage(
              title: context.l10n.renewPidVerifyingDigidTitle,
              description: context.l10n.renewPidVerifyingDigidDescription,
              contextImage: Image.asset(WalletAssets.logo_wallet, height: 64, width: 64),
            );
          case RenewPidCheckData():
            result = RenewPidCheckDetailsPage(
              attributes: state.availableAttributes,
              onAcceptPressed: () => context.bloc.add(RenewPidAttributesConfirmed(state.availableAttributes)),
              onRejectPressed: () => context.bloc.add(const RenewPidAttributesRejected()),
            );
          case RenewPidConfirmPin():
            result = RenewPidConfirmPinPage(
              onPidAccepted: (_) => context.bloc.add(RenewPidPinConfirmed()),
              onAcceptPidFailed: (BuildContext context, ErrorState state) => context.bloc.add(
                RenewPidPinConfirmationFailed(error: state.error),
              ),
            );
          case RenewPidUpdatingCards():
            result = GenericLoadingPage(
              title: context.l10n.renewPidUpdatingCardsTitle,
              description: context.l10n.renewPidUpdatingCardsDescription,
            );
          case RenewPidSuccess():
            result = RenewPidSuccessPage(
              cards: state.addedCards,
              onPrimaryPressed: () => Navigator.popUntil(context, ModalRoute.withName(WalletRoutes.dashboardRoute)),
            );
          case RenewPidDigidMismatch():
            result = RenewPidDigidMismatchPage(
              onPrimaryPressed: () => context.bloc.add(const RenewPidRetryPressed()),
              onSecondaryButtonPressed: _launchDigidWebsite,
            );
          case RenewPidDigidLoginCancelled():
            result = RenewPidLoginCancelledPage(
              onPrimaryPressed: () => context.bloc.add(const RenewPidRetryPressed()),
              onSecondaryButtonPressed: _launchDigidWebsite,
            );
          case RenewPidStopped():
            result = RenewPidStoppedPage(onPrimaryPressed: () => Navigator.pop(context));
          case RenewPidNetworkError():
            result = state.hasInternet
                ? ErrorPage.network(
                    context,
                    onPrimaryActionPressed: () => context.bloc.add(const RenewPidRetryPressed()),
                    style: ErrorCtaStyle.retry,
                  )
                : ErrorPage.noInternet(
                    context,
                    onPrimaryActionPressed: () => context.bloc.add(const RenewPidRetryPressed()),
                    style: ErrorCtaStyle.retry,
                  );
          case RenewPidDigidFailure():
          case RenewPidGenericError():
            result = ErrorPage.generic(
              context,
              onPrimaryActionPressed: () => context.bloc.add(const RenewPidLoginWithDigidClicked()),
              style: ErrorCtaStyle.retry,
            );
          case RenewPidSessionExpired():
            result = ErrorPage.sessionExpired(
              context,
              onPrimaryActionPressed: () => context.bloc.add(const RenewPidLoginWithDigidClicked()),
              style: ErrorCtaStyle.retry,
            );
        }
        return PopScope(
          canPop: _canPop(state),
          child: FakePagingAnimatedSwitcher(
            animateBackwards: state.didGoBack,
            child: result,
          ),
          onPopInvokedWithResult: (didPop, result) {
            if (!didPop) _onPopInvoked(context, state);
          },
        );
      },
    );
  }

  void _onPopInvoked(BuildContext context, RenewPidState state) {
    if (state.canGoBack) {
      context.bloc.add(const RenewPidBackPressed());
    } else {
      _stopRenewPid(context);
    }
  }

  /// Decide whether the user is allowed to pop the screen in the current state.
  /// When false the (os level) back gestures are disabled.
  bool _canPop(RenewPidState state) {
    switch (state) {
      case RenewPidInitial():
      case RenewPidSuccess():
      case RenewPidDigidFailure():
      case RenewPidDigidLoginCancelled():
      case RenewPidNetworkError():
      case RenewPidGenericError():
      case RenewPidSessionExpired():
      case RenewPidDigidMismatch():
      case RenewPidStopped():
        return true;
      case RenewPidLoadingDigidUrl():
      case RenewPidAwaitingDigidAuthentication():
      case RenewPidVerifyingDigidAuthentication():
      case RenewPidCheckData():
      case RenewPidConfirmPin():
      case RenewPidUpdatingCards():
        return false;
    }
  }

  /// Stop the renew PID flow, this methods checks the current state to make
  /// sure the correct stop action (dialog/sheet/pop) will be executed.
  Future<void> _stopRenewPid(BuildContext context) async {
    final state = context.bloc.state;
    if (state is RenewPidAwaitingDigidAuthentication) {
      // This is a special case, for which we show the stop dialog
      unawaited(_showStopDigidLoginDialog(context));
    } else {
      if (await RenewPidStopSheet.show(context) && context.mounted) {
        context.bloc.add(const RenewPidStopPressed());
      }
    }
  }

  /// Show a stop login dialog, only shown when the app is in the [RenewPidAwaitingDigidAuthentication] state.
  /// When user confirms the stop action the login with DigiD session is aborted.
  Future<void> _showStopDigidLoginDialog(BuildContext context) async {
    if (context.bloc.state is! RenewPidAwaitingDigidAuthentication) return; // Sanity check to avoid race conditions.
    final cancelled = await StopDigidLoginDialog.show(context);
    if (cancelled && context.mounted) {
      context.bloc.add(
        RenewPidLoginWithDigidFailed(
          cancelledByUser: true,
          error: GenericError('aborted_by_user', sourceError: Exception('Login cancelled')),
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
          context.bloc.add(RenewPidLaunchDigidUrlFailed(error: error));
        } else {
          Fimber.e('Failed to notify bloc', ex: error);
        }
      }
    }
  }

  /// Initiate the mock digid login and and trigger [PidRenewalNavigationRequest] on success
  Future<void> _performMockLogin(BuildContext context) async {
    assert(Environment.mockRepositories, 'Mock login is intended for mock builds only');
    final success = await MockDigidScreen.mockLogin(context);
    if (success && context.mounted) {
      await context.read<NavigationService>().handleNavigationRequest(PidRenewalNavigationRequest('renew_pid'));
    } else if (context.mounted) {
      context.bloc.add(
        const RenewPidLoginWithDigidFailed(
          error: RedirectUriError(redirectError: RedirectError.loginRequired, sourceError: 'mock'),
        ),
      );
    }
  }

  /// Launches the DigiD help page in the external browser
  void _launchDigidWebsite() => launchUrlStringCatching(kDigidWebsiteUrl, mode: LaunchMode.externalApplication);
}

extension _RenewPidScreenExtensions on BuildContext {
  RenewPidBloc get bloc => read<RenewPidBloc>();
}
