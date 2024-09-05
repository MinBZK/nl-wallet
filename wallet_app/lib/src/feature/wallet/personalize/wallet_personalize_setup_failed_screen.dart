import 'package:flutter/material.dart';

import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../common/widget/sliver_wallet_app_bar.dart';
import '../../common/widget/wallet_scrollbar.dart';

class WalletPersonalizeSetupFailedScreen extends StatelessWidget {
  const WalletPersonalizeSetupFailedScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        minimum: const EdgeInsets.only(bottom: 24),
        child: WalletScrollbar(
          child: CustomScrollView(
            slivers: [
              SliverWalletAppBar(
                title: context.l10n.walletPersonalizeSetupFailedScreenHeadline,
                scrollController: PrimaryScrollController.maybeOf(context),
              ),
              SliverToBoxAdapter(
                child: Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: Text(
                    context.l10n.walletPersonalizeSetupFailedScreenDescription,
                    textAlign: TextAlign.start,
                    style: context.textTheme.bodyLarge,
                  ),
                ),
              ),
              const SliverSizedBox(height: 24),
              SliverToBoxAdapter(
                child: ExcludeSemantics(
                  child: Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 16),
                    child: Image.asset(
                      WalletAssets.illustration_digid_failure,
                      fit: context.isLandscape ? BoxFit.contain : BoxFit.fitWidth,
                      height: context.isLandscape ? 160 : null,
                      width: double.infinity,
                    ),
                  ),
                ),
              ),
              const SliverSizedBox(height: 32),
              SliverFillRemaining(
                hasScrollBody: false,
                fillOverscroll: true,
                child: _buildBottomSection(context),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Container(
      padding: const EdgeInsets.only(top: 24, left: 16, right: 16),
      alignment: Alignment.bottomCenter,
      child: ElevatedButton(
        onPressed: () async {
          final navigator = Navigator.of(context);
          await navigator.pushNamedAndRemoveUntil(
            WalletRoutes.setupSecurityRoute,
            ModalRoute.withName(WalletRoutes.splashRoute),
          );
        },
        child: Text.rich(context.l10n.walletPersonalizeSetupFailedScreenCta.toTextSpan(context)),
      ),
    );
  }

  static void show(BuildContext context) {
    Navigator.pushAndRemoveUntil(
      context,
      MaterialPageRoute(builder: (c) => const WalletPersonalizeSetupFailedScreen()),
      ModalRoute.withName(WalletRoutes.splashRoute),
    );
  }
}
