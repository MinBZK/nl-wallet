import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/usecase/wallet/is_wallet_initialized_with_pid_usecase.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/cast_util.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/list_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../wallet_assets.dart';
import '../common/dialog/reset_wallet_dialog.dart';
import '../common/widget/button/icon/back_icon_button.dart';
import '../common/widget/button/icon/close_icon_button.dart';
import '../common/widget/button/primary_button.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/page_illustration.dart';
import '../common/widget/paragraphed_list.dart';
import '../common/widget/spacer/sliver_sized_box.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';
import 'argument/forgot_pin_screen_argument.dart';

class ForgotPinScreen extends StatelessWidget {
  /// When set, uses 'close' copy instead of the default 'back' behaviour.
  final bool useCloseButton;

  const ForgotPinScreen({this.useCloseButton = false, super.key});

  static ForgotPinScreenArgument getArgument(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return tryCast<ForgotPinScreenArgument>(args) ?? ForgotPinScreenArgument.fromJson(args! as Map<String, dynamic>);
    } catch (exception) {
      Fimber.i('Failed to decode $args, falling back to default', ex: exception);
      return const ForgotPinScreenArgument(useCloseButton: false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(context.l10n.forgotPinScreenTitle),
        automaticallyImplyLeading: false,
        leading: useCloseButton ? null : const BackIconButton(),
        actions: [useCloseButton ? const CloseIconButton() : null].nonNullsList,
      ),
      key: const Key('forgotPinScreen'),
      body: SafeArea(
        child: FutureBuilder(
          future: _getPinRecoveryMethod(context),
          builder: (context, recoveryMethodSnapshot) {
            final recoveryMethod = recoveryMethodSnapshot.data;
            if (recoveryMethod == null) return const CenteredLoadingIndicator();
            return Column(
              children: [
                Expanded(child: _buildScrollableSection(context, recoveryMethod)),
                _buildBottomSection(context, recoveryMethod),
              ],
            );
          },
        ),
      ),
    );
  }

  Widget _buildScrollableSection(BuildContext context, PinRecoveryMethod method) {
    final content = switch (method) {
      PinRecoveryMethod.recoverPin => context.l10n.forgotPinScreenDescription,
      PinRecoveryMethod.resetWallet => context.l10n.forgotPinScreenResetDescription,
    };
    return WalletScrollbar(
      child: CustomScrollView(
        slivers: [
          const SliverSizedBox(height: 12),
          SliverPadding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            sliver: SliverList.list(
              children: [
                TitleText(context.l10n.forgotPinScreenTitle),
                const SizedBox(height: 8),
                ParagraphedList.splitContent(content),
                const SizedBox(height: 32),
                const PageIllustration(
                  asset: WalletAssets.svg_pin_forgot,
                  padding: EdgeInsets.zero,
                ),
              ],
            ),
          ),
          const SliverSizedBox(height: 24),
        ],
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context, PinRecoveryMethod method) {
    final cta = switch (method) {
      PinRecoveryMethod.resetWallet => _buildPinResetButton(context),
      PinRecoveryMethod.recoverPin => _buildPinRecoveryButton(context),
    };
    return Column(
      children: [
        const Divider(),
        Padding(
          padding: EdgeInsets.symmetric(horizontal: 16, vertical: context.isLandscape ? 8 : 24),
          child: Column(
            children: [
              cta,
              const SizedBox(height: 12),
              TertiaryButton(
                onPressed: () => Navigator.maybePop(context),
                text: Text.rich(
                  useCloseButton
                      ? context.l10n.generalClose.toTextSpan(context)
                      : context.l10n.generalBottomBackCta.toTextSpan(context),
                ),
                icon: Icon(useCloseButton ? Icons.close_outlined : Icons.arrow_back),
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildPinRecoveryButton(BuildContext context) {
    return PrimaryButton(
      onPressed: () => Navigator.pushNamed(context, WalletRoutes.pinRecoveryRoute),
      text: Text.rich(context.l10n.forgotPinScreenCta.toTextSpan(context)),
    );
  }

  Widget _buildPinResetButton(BuildContext context) {
    return PrimaryButton(
      onPressed: () => ResetWalletDialog.show(context),
      icon: const Icon(Icons.delete_outline_rounded),
      text: Text.rich(context.l10n.forgotPinScreenResetCta.toTextSpan(context)),
    );
  }

  Future<PinRecoveryMethod> _getPinRecoveryMethod(BuildContext context) async {
    final isInitializedWithPid = await context.read<IsWalletInitializedWithPidUseCase>().invoke();
    return isInitializedWithPid ? PinRecoveryMethod.recoverPin : PinRecoveryMethod.resetWallet;
  }

  static Future<void> show(BuildContext context, {bool popToDashboard = false}) {
    if (popToDashboard) {
      return Navigator.pushNamedAndRemoveUntil(
        context,
        WalletRoutes.forgotPinRoute,
        ModalRoute.withName(WalletRoutes.dashboardRoute),
        arguments: ForgotPinScreenArgument(useCloseButton: popToDashboard).toJson(),
      );
    } else {
      return Navigator.pushNamed(
        context,
        WalletRoutes.forgotPinRoute,
        arguments: ForgotPinScreenArgument(useCloseButton: popToDashboard).toJson(),
      );
    }
  }
}

/// Specify the recovery path that is available to the user
enum PinRecoveryMethod { resetWallet, recoverPin }
