import 'package:flutter/material.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../common/screen/placeholder_screen.dart';
import '../common/widget/bullet_list.dart';
import '../common/widget/button/text_icon_button.dart';
import '../common/widget/button/wallet_back_button.dart';
import '../common/widget/sliver_wallet_app_bar.dart';

class IntroductionPrivacyScreen extends StatelessWidget {
  const IntroductionPrivacyScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('introductionPrivacyScreen'),
      body: SafeArea(child: _buildContent(context)),
    );
  }

  Widget _buildContent(BuildContext context) {
    return Scrollbar(
      thumbVisibility: true,
      child: CustomScrollView(
        slivers: [
          SliverWalletAppBar(
            title: context.l10n.introductionPrivacyScreenHeadline,
            leading: const WalletBackButton(),
            progress: 0.08,
            actions: [
              IconButton(
                onPressed: () => Navigator.pushNamed(context, WalletRoutes.aboutRoute),
                icon: const Icon(Icons.help_outline_rounded),
              ),
            ],
          ),
          SliverPadding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            sliver: SliverToBoxAdapter(
              child: MergeSemantics(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    BulletList(
                      items: context.l10n.introductionPrivacyScreenBulletPoints.split('\n'),
                    ),
                  ],
                ),
              ),
            ),
          ),
          SliverPadding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            sliver: SliverToBoxAdapter(
              child: Image.asset(
                WalletAssets.illustration_privacy_policy_screen,
                fit: context.isLandscape ? BoxFit.contain : BoxFit.fitWidth,
                height: context.isLandscape ? 160 : null,
                width: double.infinity,
              ),
            ),
          ),
          SliverFillRemaining(
            hasScrollBody: false,
            fillOverscroll: true,
            child: _buildBottomSection(context),
          )
        ],
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Padding(
      padding: EdgeInsets.symmetric(horizontal: 16, vertical: context.isLandscape ? 8 : 24),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        mainAxisAlignment: MainAxisAlignment.end,
        children: [
          TextIconButton(
            key: const Key('introductionPrivacyScreenPrivacyCta'),
            iconPosition: IconPosition.start,
            child: Text(context.l10n.introductionPrivacyScreenPrivacyCta),
            onPressed: () => PlaceholderScreen.show(context, secured: false),
          ),
          const SizedBox(height: 8),
          ElevatedButton(
            key: const Key('introductionPrivacyScreenNextCta'),
            onPressed: () => Navigator.of(context).restorablePushNamed(WalletRoutes.introductionConditionsRoute),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Icon(Icons.arrow_forward, size: 16),
                const SizedBox(width: 8),
                Text(context.l10n.introductionPrivacyScreenNextCta),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
