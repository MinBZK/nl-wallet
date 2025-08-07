import 'package:flutter/material.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../wallet_assets.dart';
import '../common/dialog/reset_wallet_dialog.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/button/primary_button.dart';
import '../common/widget/page_illustration.dart';
import '../common/widget/spacer/sliver_sized_box.dart';
import '../common/widget/text/body_text.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';

class PinBlockedScreen extends StatelessWidget {
  const PinBlockedScreen({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(context.l10n.pinBlockedScreenHeadline),
        actions: const [HelpIconButton()],
      ),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: WalletScrollbar(
                child: CustomScrollView(
                  slivers: [
                    SliverPadding(
                      padding: const EdgeInsets.symmetric(horizontal: 16),
                      sliver: SliverToBoxAdapter(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            const SizedBox(height: 12),
                            TitleText(context.l10n.pinBlockedScreenHeadline),
                            const SizedBox(height: 8),
                            BodyText(context.l10n.pinBlockedScreenDescription),
                          ],
                        ),
                      ),
                    ),
                    const SliverSizedBox(height: 24),
                    const SliverPadding(
                      padding: EdgeInsets.symmetric(horizontal: 16),
                      sliver: SliverToBoxAdapter(
                        child: PageIllustration(
                          asset: WalletAssets.svg_blocked_final,
                          padding: EdgeInsets.zero,
                        ),
                      ),
                    ),
                    const SliverSizedBox(height: 24),
                  ],
                ),
              ),
            ),
            const Divider(),
            SizedBox(height: context.orientationBasedVerticalPadding),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: PrimaryButton(
                icon: const Icon(Icons.delete_outline_rounded),
                text: Text.rich(context.l10n.pinBlockedScreenResetWalletCta.toTextSpan(context)),
                onPressed: () => ResetWalletDialog.show(context),
              ),
            ),
            SizedBox(height: context.orientationBasedVerticalPadding),
          ],
        ),
      ),
    );
  }

  static void show(BuildContext context) {
    // Remove all routes and only keep the pinBlocked route
    Navigator.pushNamedAndRemoveUntil(context, WalletRoutes.pinBlockedRoute, (Route<dynamic> route) => false);
  }
}
