import 'dart:io';

import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/model/bloc/error_state.dart';
import '../../util/extension/build_context_extension.dart';
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
import '../common/widget/text/title_text.dart';
import '../common/widget/utility/scroll_offset_provider.dart';
import '../common/widget/wallet_app_bar.dart';
import '../error/error_page.dart';
import 'bloc/wallet_transfer_source_bloc.dart';
import 'page/wallet_transfer_source_confirm_pin_page.dart';
import 'page/wallet_transfer_source_transfer_success_page.dart';
import 'page/wallet_transfer_source_transferring_page.dart';
import 'widget/wallet_transfer_source_stop_sheet.dart';

class WalletTransferSourceScreen extends StatelessWidget {
  const WalletTransferSourceScreen({super.key});

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
        body: BlocConsumer<WalletTransferSourceBloc, WalletTransferSourceState>(
          listener: (context, state) {
            context.read<ScrollOffset>().reset(); // Rest the scrollOffset used by fading title on page transitions
            DialogHelper.dismissOpenDialogs(context); // Dismiss potentially open stop sheet on page transitions
          },
          builder: (context, state) {
            void pop() => Navigator.pop(context);
            final Widget page = switch (state) {
              WalletTransferInitial() => GenericLoadingPage(
                title: context.l10n.walletTransferScreenLoadingTitle,
                description: context.l10n.walletTransferScreenLoadingDescription,
                onCancel: () => _onStopPressed(context),
              ),
              WalletTransferLoading() => GenericLoadingPage(
                title: context.l10n.walletTransferScreenLoadingTitle,
                description: context.l10n.walletTransferScreenLoadingDescription,
                onCancel: () => _onStopPressed(context),
              ),
              WalletTransferIntroduction() => TerminalPage(
                title: context.l10n.walletTransferSourceScreenIntroductionTitle,
                description: context.l10n.walletTransferSourceScreenIntroductionDescription,
                primaryButtonCta: context.l10n.walletTransferSourceScreenIntroductionCta,
                illustration: const PageIllustration(asset: WalletAssets.svg_move_source_confirm),
                onPrimaryPressed: () => context.bloc.add(const WalletTransferAgreeEvent()),
                secondaryButtonCta: context.l10n.generalStop,
                secondaryButtonIcon: const Icon(Icons.block_flipped),
                onSecondaryButtonPressed: () => _onStopPressed(context),
              ),
              WalletTransferConfirmPin() => WalletTransferSourceConfirmPinPage(
                onPinConfirmed: (_) => context.bloc.add(const WalletTransferPinConfirmedEvent()),
                onPinConfirmationFailed: (BuildContext context, ErrorState state) =>
                    context.bloc.add(WalletTransferPinConfirmationFailed(state.error)),
              ),
              WalletTransferTransferring() => WalletTransferSourceTransferringPage(
                onStopPressed: () => _onStopPressed(context),
              ),
              WalletTransferSuccess() => const WalletTransferSourceTransferSuccessPage(),
              WalletTransferStopped() => TerminalPage(
                title: context.l10n.walletTransferScreenStoppedTitle,
                description: context.l10n.walletTransferSourceScreenStoppedDescription,
                onPrimaryPressed: pop,
                primaryButtonCta: context.l10n.generalClose,
                primaryButtonIcon: const Icon(Icons.close_outlined),
                illustration: const PageIllustration(asset: WalletAssets.svg_stopped),
              ),
              WalletTransferGenericError() => ErrorPage.generic(
                context,
                onPrimaryActionPressed: pop,
                style: ErrorCtaStyle.close,
              ),
              WalletTransferNetworkError() =>
                state.hasInternet
                    ? ErrorPage.network(context, onPrimaryActionPressed: pop, style: ErrorCtaStyle.close)
                    : ErrorPage.noInternet(context, onPrimaryActionPressed: pop, style: ErrorCtaStyle.close),
              WalletTransferSessionExpired() => ErrorPage.sessionExpired(
                context,
                style: ErrorCtaStyle.close,
                onPrimaryActionPressed: pop,
              ),
              WalletTransferFailed() => TerminalPage(
                title: context.l10n.walletTransferScreenFailedTitle,
                description: context.l10n.walletTransferScreenFailedDescription,
                illustration: const PageIllustration(asset: WalletAssets.svg_error_general),
                flipButtonOrder: true,
                primaryButtonCta: context.l10n.generalClose,
                primaryButtonIcon: const Icon(Icons.close),
                onPrimaryPressed: pop,
                secondaryButtonCta: context.l10n.generalShowDetailsCta,
                secondaryButtonIcon: const Icon(Icons.info_outline_rounded),
                onSecondaryButtonPressed: () => ErrorDetailsSheet.show(context, error: state.error),
              ),
            };
            return FakePagingAnimatedSwitcher(
              animateBackwards: state.didGoBack,
              child: PopScope(
                key: ValueKey(state.runtimeType),
                canPop: _canPop(state),
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
    final canGoBack = context.watch<WalletTransferSourceBloc>().state.canGoBack;
    if (!canGoBack) return null;
    return BackIconButton(onPressed: () => context.bloc.add(const WalletTransferBackPressedEvent()));
  }

  Widget _buildCloseButton(BuildContext context) {
    final state = context.watch<WalletTransferSourceBloc>().state;
    switch (state) {
      case WalletTransferInitial():
      case WalletTransferLoading():
      case WalletTransferIntroduction():
      case WalletTransferConfirmPin():
        return CloseIconButton(onPressed: () => _onStopPressed(context));
      case WalletTransferTransferring():
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
    final stopConfirmed = await WalletTransferSourceStopSheet.show(context);
    if (stopConfirmed && context.mounted) context.bloc.add(const WalletTransferStopRequestedEvent());
  }

  bool _canPop(WalletTransferSourceState state) {
    switch (state) {
      case WalletTransferInitial():
      case WalletTransferStopped():
      case WalletTransferGenericError():
      case WalletTransferNetworkError():
      case WalletTransferSessionExpired():
      case WalletTransferFailed():
        return true;
      case WalletTransferIntroduction():
      case WalletTransferLoading():
      case WalletTransferConfirmPin():
      case WalletTransferTransferring():
      case WalletTransferSuccess():
        return false;
    }
  }

  void _onPopInvoked(BuildContext context, WalletTransferSourceState state) {
    switch (state) {
      case WalletTransferInitial():
      case WalletTransferLoading():
      case WalletTransferIntroduction():
      case WalletTransferTransferring():
        _onStopPressed(context);
      case WalletTransferConfirmPin():
        context.bloc.add(const WalletTransferBackPressedEvent());
      case WalletTransferSuccess():
        if (Platform.isAndroid) SystemNavigator.pop();
      default:
        Fimber.d('Unhandled onPopInvoked for state: $state');
    }
  }

  Widget _buildTitle(BuildContext context) {
    final state = context.watch<WalletTransferSourceBloc>().state;
    return switch (state) {
      WalletTransferInitial() => const SizedBox.shrink(),
      WalletTransferLoading() => const SizedBox.shrink(),
      WalletTransferIntroduction() => TitleText(context.l10n.walletTransferSourceScreenIntroductionTitle),
      WalletTransferConfirmPin() => const SizedBox.shrink(),
      WalletTransferTransferring() => TitleText(context.l10n.walletTransferSourceScreenTransferringTitle),
      WalletTransferSuccess() => TitleText(context.l10n.walletTransferSourceScreenSuccessTitle),
      WalletTransferStopped() => TitleText(context.l10n.walletTransferScreenStoppedTitle),
      WalletTransferGenericError() => TitleText(context.l10n.errorScreenGenericHeadline),
      WalletTransferNetworkError() =>
        state.hasInternet
            ? TitleText(context.l10n.errorScreenServerHeadline)
            : TitleText(context.l10n.errorScreenNoInternetHeadline),
      WalletTransferSessionExpired() => TitleText(context.l10n.errorScreenSessionExpiredHeadline),
      WalletTransferFailed() => TitleText(context.l10n.walletTransferScreenFailedTitle),
    };
  }
}

extension _WalletTransferSourceScreenExtension on BuildContext {
  WalletTransferSourceBloc get bloc => read();
}
