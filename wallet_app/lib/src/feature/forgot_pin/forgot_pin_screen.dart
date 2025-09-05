import 'package:flutter/material.dart';

import '../../../environment.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../wallet_assets.dart';
import '../common/dialog/reset_wallet_dialog.dart';
import '../common/widget/button/primary_button.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/page_illustration.dart';
import '../common/widget/paragraphed_list.dart';
import '../common/widget/spacer/sliver_sized_box.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';

class ForgotPinScreen extends StatelessWidget {
  const ForgotPinScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(context.l10n.forgotPinScreenTitle),
      ),
      key: const Key('forgotPinScreen'),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(child: _buildScrollableSection(context)),
            _buildBottomSection(context),
          ],
        ),
      ),
    );
  }

  Widget _buildScrollableSection(BuildContext context) {
    return WalletScrollbar(
      child: CustomScrollView(
        slivers: [
          const SliverSizedBox(height: 12),
          SliverPadding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            sliver: _buildContentSliver(context),
          ),
          const SliverSizedBox(height: 24),
        ],
      ),
    );
  }

  Widget _buildContentSliver(BuildContext context) {
    return SliverList.list(
      children: [
        TitleText(context.l10n.forgotPinScreenTitle),
        const SizedBox(height: 8),
        // TODO(Rob): Remove legacy flow. Awaiting core implementation. PVW-4587
        ParagraphedList.splitContent(
          Environment.isMockOrTest
              ? context.l10n.forgotPinScreenDescription
              : context.l10n.forgotPinScreenLegacyDescription,
        ),
        const SizedBox(height: 32),
        const PageIllustration(
          asset: WalletAssets.svg_pin_forgot,
          padding: EdgeInsets.zero,
        ),
      ],
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Column(
      children: [
        const Divider(),
        Padding(
          padding: EdgeInsets.symmetric(horizontal: 16, vertical: context.isLandscape ? 8 : 24),
          child: Column(
            children: [
              // TODO(Rob): Remove legacy flow. Awaiting core implementation. PVW-4587
              Environment.isMockOrTest ? _buildPinRecoveryButton(context) : _buildPinResetButton(context),
              const SizedBox(height: 12),
              TertiaryButton(
                onPressed: () => Navigator.maybePop(context),
                text: Text.rich(context.l10n.generalBottomBackCta.toTextSpan(context)),
                icon: const Icon(Icons.arrow_back),
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
      text: Text.rich(context.l10n.forgotPinScreenLegacyCta.toTextSpan(context)),
    );
  }

  static void show(BuildContext context) => Navigator.pushNamed(context, WalletRoutes.forgotPinRoute);
}
