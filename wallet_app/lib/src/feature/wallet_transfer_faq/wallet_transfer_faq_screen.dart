import 'package:flutter/material.dart';

import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../common/widget/bullet_list.dart';
import '../common/widget/bullet_list_dot.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/icon/back_icon_button.dart';
import '../common/widget/page_illustration.dart';
import '../common/widget/text/body_text.dart';
import '../common/widget/text/headline_small_text.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';

class WalletTransferFaqScreen extends StatelessWidget {
  const WalletTransferFaqScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(context.l10n.walletTransferFaqScreenTitle),
        leading: const BackIconButton(),
      ),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: ListView(
                children: [
                  Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 16),
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        TitleText(context.l10n.walletTransferFaqScreenTitle),
                        const SizedBox(height: 24),
                        HeadlineSmallText(
                          context.l10n.walletTransferFaqScreenSection1Heading,
                          style: context.textTheme.labelMedium,
                        ),
                        const SizedBox(height: 4),
                        BulletList(
                          items: context.l10n.walletTransferFaqScreenSection1Content.split('\n'),
                          icon: const BulletListDot(),
                        ),
                        const SizedBox(height: 16),
                        HeadlineSmallText(
                          context.l10n.walletTransferFaqScreenSection2Heading,
                          style: context.textTheme.labelMedium,
                        ),
                        const SizedBox(height: 4),
                        BodyText(context.l10n.walletTransferFaqScreenSection2Content),
                        const SizedBox(height: 4),
                        BodyText(context.l10n.walletTransferFaqScreenFooter),
                        const SizedBox(height: 16),
                      ],
                    ),
                  ),
                  const PageIllustration(asset: WalletAssets.svg_move_source_confirm),
                ],
              ),
            ),
            const BottomBackButton(),
          ],
        ),
      ),
    );
  }
}
