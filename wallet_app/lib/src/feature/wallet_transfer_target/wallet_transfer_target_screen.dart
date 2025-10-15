import 'dart:async';

import 'package:after_layout/after_layout.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../data/service/navigation_service.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../util/helper/dialog_helper.dart';
import '../../wallet_assets.dart';
import '../common/page/generic_loading_page.dart';
import '../common/page/terminal_page.dart';
import '../common/sheet/error_details_sheet.dart';
import '../common/widget/button/icon/back_icon_button.dart';
import '../common/widget/button/icon/close_icon_button.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/fake_paging_animated_switcher.dart';
import '../common/widget/page_illustration.dart';
import '../common/widget/text/body_text.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/utility/scroll_offset_provider.dart';
import '../common/widget/wallet_app_bar.dart';
import '../dashboard/dashboard_screen.dart';
import '../error/error_page.dart';
import 'bloc/wallet_transfer_target_bloc.dart';
import 'page/wallet_transfer_awaiting_confirmation_page.dart';
import 'page/wallet_transfer_awaiting_scan_page.dart';
import 'page/wallet_transfer_target_transferring_page.dart';
import 'widget/wallet_transfer_target_stop_sheet.dart';

class WalletTransferTargetScreen extends StatefulWidget {
  const WalletTransferTargetScreen({super.key});

  @override
  State<WalletTransferTargetScreen> createState() => _WalletTransferTargetScreenState();
}

class _WalletTransferTargetScreenState extends State<WalletTransferTargetScreen>
    with AfterLayoutMixin<WalletTransferTargetScreen> {
  @override
  FutureOr<void> afterFirstLayout(BuildContext context) {
    final showRecoveryPopup = ModalRoute.of(context)?.settings.arguments == true;
    if (showRecoveryPopup) context.read<NavigationService>().showDialog(WalletDialogType.moveStopped);
  }

  @override
  Widget build(BuildContext context) {
    return ScrollOffsetProvider(
      child: Scaffold(
        appBar: WalletAppBar(
          title: _buildTitle(context),
          leading: _buildBackButton(context),
          automaticallyImplyLeading: false,
          actions: [const HelpIconButton(), _buildCloseButton(context)],
          progress: context.bloc.state.stepperProgress,
        ),
        body: BlocConsumer<WalletTransferTargetBloc, WalletTransferTargetState>(
          listener: (context, state) {
            context.read<ScrollOffset>().reset(); // Rest the scrollOffset used by fading title on page transitions
            DialogHelper.dismissOpenDialogs(context); // Dismiss potentially open stop sheet on page transitions
          },
          builder: (context, state) {
            void restart() => context.bloc.add(const WalletTransferRestartEvent());
            final Widget page = switch (state) {
              WalletTransferIntroduction() => TerminalPage(
                title: context.l10n.walletTransferTargetScreenIntroductionTitle,
                description: context.l10n.walletTransferTargetScreenIntroductionDescription,
                primaryButtonCta: context.l10n.walletTransferTargetScreenIntroductionOptInCta,
                primaryButtonIcon: const Icon(Icons.arrow_forward_outlined),
                illustration: const PageIllustration(asset: WalletAssets.svg_move_source_confirm),
                onPrimaryPressed: () => context.bloc.add(const WalletTransferOptInEvent()),
                secondaryButtonCta: context.l10n.walletTransferTargetScreenIntroductionOptOutCta,
                secondaryButtonIcon: const Icon(Icons.arrow_forward_outlined),
                onSecondaryButtonPressed: () => _onSkipPressed(context),
              ),
              WalletTransferLoadingQrData() => GenericLoadingPage(
                title: context.l10n.walletTransferLoadingQrTitle,
                description: context.l10n.walletTransferLoadingQrDescription,
                onCancel: () => _onStopPressed(context),
              ),
              WalletTransferAwaitingQrScan() => WalletTransferAwaitingScanPage(
                data: state.qrContents,
                onBackPressed: () => context.bloc.add(const WalletTransferBackPressedEvent()),
              ),
              WalletTransferAwaitingConfirmation() => WalletTransferAwaitingConfirmationPage(
                onCtaPressed: () => _onStopPressed(context),
              ),
              WalletTransferTransferring() => WalletTransferTargetTransferringPage(
                onStopPressed: () => _onStopPressed(context),
              ),
              WalletTransferSuccess() => TerminalPage(
                title: context.l10n.walletTransferTargetScreenSuccessTitle,
                description: context.l10n.walletTransferTargetScreenSuccessDescription,
                primaryButtonCta: context.l10n.walletTransferTargetScreenSuccessCta,
                illustration: const PageIllustration(asset: WalletAssets.svg_move_destination_success),
                onPrimaryPressed: () => DashboardScreen.show(context),
              ),
              WalletTransferStopped() => TerminalPage(
                title: context.l10n.walletTransferScreenStoppedTitle,
                description: context.l10n.walletTransferTargetScreenStoppedDescription,
                onPrimaryPressed: restart,
                primaryButtonCta: context.l10n.generalRetry,
                primaryButtonIcon: const Icon(Icons.refresh_outlined),
                illustration: const PageIllustration(asset: WalletAssets.svg_stopped),
              ),
              WalletTransferGenericError() => ErrorPage.generic(
                context,
                onPrimaryActionPressed: restart,
                style: ErrorCtaStyle.retry,
              ),
              WalletTransferNetworkError() =>
                state.hasInternet
                    ? ErrorPage.network(context, onPrimaryActionPressed: restart, style: ErrorCtaStyle.retry)
                    : ErrorPage.noInternet(context, onPrimaryActionPressed: restart, style: ErrorCtaStyle.retry),
              WalletTransferSessionExpired() => ErrorPage.sessionExpired(
                context,
                onPrimaryActionPressed: restart,
                style: ErrorCtaStyle.retry,
              ),
              WalletTransferFailed() => TerminalPage(
                title: context.l10n.walletTransferScreenFailedTitle,
                description: context.l10n.walletTransferScreenFailedDescription,
                illustration: const PageIllustration(asset: WalletAssets.svg_error_general),
                flipButtonOrder: true,
                primaryButtonCta: context.l10n.generalRetry,
                primaryButtonIcon: const Icon(Icons.refresh_outlined),
                onPrimaryPressed: restart,
                secondaryButtonCta: context.l10n.generalShowDetailsCta,
                secondaryButtonIcon: const Icon(Icons.info_outline_rounded),
                onSecondaryButtonPressed: () => ErrorDetailsSheet.show(context, error: state.error),
              ),
            };
            return FakePagingAnimatedSwitcher(
              animateBackwards: state.didGoBack,
              child: PopScope(
                key: ValueKey(state.runtimeType),
                canPop: false,
                onPopInvokedWithResult: (didPop, result) {
                  if (!didPop) _onPopInvoked(context, state);
                },
                child: page,
              ),
            );
          },
        ),
      ),
    );
  }

  Widget? _buildBackButton(BuildContext context) {
    final canGoBack = context.watch<WalletTransferTargetBloc>().state.canGoBack;
    if (!canGoBack) return null;
    return BackIconButton(onPressed: () => context.bloc.add(const WalletTransferBackPressedEvent()));
  }

  Widget _buildCloseButton(BuildContext context) {
    final state = context.watch<WalletTransferTargetBloc>().state;
    switch (state) {
      case WalletTransferIntroduction():
        return CloseIconButton(onPressed: () => _onSkipPressed(context));
      case WalletTransferAwaitingConfirmation():
        return CloseIconButton(onPressed: () => _onStopPressed(context));
      case WalletTransferTransferring():
      case WalletTransferLoadingQrData():
      case WalletTransferAwaitingQrScan():
      case WalletTransferSuccess():
      case WalletTransferStopped():
      case WalletTransferGenericError():
      case WalletTransferNetworkError():
      case WalletTransferSessionExpired():
      case WalletTransferFailed():
        return const SizedBox.shrink();
    }
  }

  Future<void> _onStopPressed(BuildContext context) async {
    final stopConfirmed = await WalletTransferTargetStopSheet.show(context);
    if (stopConfirmed && context.mounted) context.bloc.add(const WalletTransferStopRequestedEvent());
  }

  void _onPopInvoked(BuildContext context, WalletTransferTargetState state) {
    switch (state) {
      case WalletTransferLoadingQrData():
      case WalletTransferAwaitingConfirmation():
      case WalletTransferTransferring():
        _onStopPressed(context);
      case WalletTransferAwaitingQrScan():
        context.bloc.add(const WalletTransferBackPressedEvent());
      case WalletTransferSuccess():
        DashboardScreen.show(context);
      default:
        Fimber.d('Unhandled onPopInvoked for state: $state');
    }
  }

  Widget _buildTitle(BuildContext context) {
    final state = context.watch<WalletTransferTargetBloc>().state;
    return switch (state) {
      WalletTransferAwaitingQrScan() => TitleText(context.l10n.walletTransferAwaitingScanPageTitle),
      WalletTransferIntroduction() => TitleText(context.l10n.walletTransferSourceScreenIntroductionTitle),
      WalletTransferAwaitingConfirmation() => TitleText(context.l10n.walletTransferAwaitingConfirmationPageTitle),
      WalletTransferTransferring() => TitleText(context.l10n.walletTransferTargetScreenTransferringTitle),
      WalletTransferSuccess() => TitleText(context.l10n.walletTransferTargetScreenSuccessTitle),
      WalletTransferStopped() => TitleText(context.l10n.walletTransferScreenStoppedTitle),
      WalletTransferGenericError() => TitleText(context.l10n.errorScreenGenericHeadline),
      WalletTransferNetworkError() =>
        state.hasInternet
            ? TitleText(context.l10n.errorScreenServerHeadline)
            : TitleText(context.l10n.errorScreenNoInternetHeadline),
      WalletTransferSessionExpired() => TitleText(context.l10n.errorScreenSessionExpiredHeadline),
      WalletTransferFailed() => TitleText(context.l10n.walletTransferScreenFailedTitle),
      WalletTransferLoadingQrData() => const SizedBox.shrink(),
    };
  }

  Future<void> _onSkipPressed(BuildContext context) async {
    final skip =
        await showDialog<bool>(
          context: context,
          builder: (context) => AlertDialog(
            title: TitleText(context.l10n.walletTransferTargetOptOutDialogTitle),
            content: BodyText(context.l10n.walletTransferTargetOptOutDialogDescription),
            actions: [
              TextButton(
                child: Text.rich(context.l10n.generalCancelCta.toTextSpan(context)),
                onPressed: () => Navigator.pop(context, false),
              ),
              TextButton(
                child: Text.rich(context.l10n.generalOkCta.toTextSpan(context)),
                onPressed: () => Navigator.pop(context, true),
              ),
            ],
          ),
        ) ??
        false;

    if (skip && context.mounted) {
      context.bloc.add(const WalletTransferOptOutEvent());
      Navigator.pop(context);
    }
  }
}

extension _WalletTransferTargetScreenExtension on BuildContext {
  WalletTransferTargetBloc get bloc => read();
}
