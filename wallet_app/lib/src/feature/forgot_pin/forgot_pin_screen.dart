import 'package:flutter/material.dart';

import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../wallet_assets.dart';
import '../common/dialog/reset_wallet_dialog.dart';
import '../common/page/page_illustration.dart';
import '../common/widget/button/primary_button.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/paragraphed_list.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';

class ForgotPinScreen extends StatelessWidget {
  const ForgotPinScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
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
          SliverWalletAppBar(
            title: context.l10n.forgotPinScreenTitle,
            scrollController: PrimaryScrollController.maybeOf(context),
          ),
          SliverPadding(
            padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
            sliver: _buildContentSliver(context),
          ),
        ],
      ),
    );
  }

  Widget _buildContentSliver(BuildContext context) {
    return SliverList.list(
      children: [
        ParagraphedList.splitContent(context.l10n.forgotPinScreenDescription),
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
        const Divider(height: 1),
        Padding(
          padding: EdgeInsets.symmetric(horizontal: 16, vertical: context.isLandscape ? 8 : 24),
          child: Column(
            children: [
              PrimaryButton(
                onPressed: () => ResetWalletDialog.show(context),
                text: Text.rich(context.l10n.forgotPinScreenCta.toTextSpan(context)),
              ),
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

  static void show(BuildContext context) {
    Navigator.push(
      context,
      MaterialPageRoute(builder: (c) => const ForgotPinScreen()),
    );
  }
}
