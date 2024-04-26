import 'package:flutter/material.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../common/dialog/reset_wallet_dialog.dart';
import '../common/page/page_illustration.dart';
import '../common/widget/sliver_sized_box.dart';
import '../common/widget/wallet_app_bar.dart';

class PinBlockedScreen extends StatelessWidget {
  const PinBlockedScreen({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: Text(context.l10n.pinBlockedScreenTitle),
        automaticallyImplyLeading: false,
      ),
      body: SafeArea(
        child: PrimaryScrollController(
          controller: ScrollController(),
          child: Scrollbar(
            thumbVisibility: true,
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: CustomScrollView(
                slivers: [
                  const SliverSizedBox(height: 24),
                  const SliverToBoxAdapter(
                    child: PageIllustration(
                      asset: WalletAssets.svg_blocked_final,
                      padding: EdgeInsets.zero,
                    ),
                  ),
                  const SliverSizedBox(height: 24),
                  SliverToBoxAdapter(
                    child: Text(
                      context.l10n.pinBlockedScreenHeadline,
                      textAlign: TextAlign.start,
                      style: context.textTheme.displayMedium,
                    ),
                  ),
                  const SliverSizedBox(height: 8),
                  SliverToBoxAdapter(
                    child: Text(context.l10n.pinBlockedScreenDescription),
                  ),
                  SliverFillRemaining(
                    hasScrollBody: false,
                    fillOverscroll: true,
                    child: _buildBottomSection(context),
                  )
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Container(
      alignment: Alignment.bottomCenter,
      padding: const EdgeInsets.symmetric(vertical: 24),
      child: ElevatedButton(
        onPressed: () => ResetWalletDialog.show(context),
        child: Text(context.l10n.pinBlockedScreenResetWalletCta),
      ),
    );
  }

  static void show(BuildContext context) {
    // Remove all routes and only keep the pinBlocked route
    Navigator.pushNamedAndRemoveUntil(context, WalletRoutes.pinBlockedRoute, (Route<dynamic> route) => false);
  }
}
