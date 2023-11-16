import 'package:flutter/material.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../common/screen/placeholder_screen.dart';
import '../common/widget/bullet_list.dart';
import '../common/widget/button/text_icon_button.dart';

class IntroductionConditionsScreen extends StatelessWidget {
  const IntroductionConditionsScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('introductionConditionsScreen'),
      appBar: AppBar(
        title: Text(context.l10n.introductionConditionsScreenTitle),
      ),
      body: SafeArea(
        child: Column(
          children: [
            const LinearProgressIndicator(value: 0.16),
            Expanded(child: _buildContent(context)),
          ],
        ),
      ),
    );
  }

  Widget _buildContent(BuildContext context) {
    return Scrollbar(
      thumbVisibility: true,
      child: CustomScrollView(
        slivers: [
          SliverPadding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 32),
            sliver: SliverToBoxAdapter(
              child: MergeSemantics(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      context.l10n.introductionConditionsScreenHeadline,
                      style: context.textTheme.displayMedium,
                      textAlign: TextAlign.start,
                    ),
                    BulletList(
                      items: context.l10n.introductionConditionsScreenBulletPoints.split('\n'),
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
                WalletAssets.illustration_conditions_screen,
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
            key: const Key('introductionConditionsScreenConditionsCta'),
            iconPosition: IconPosition.start,
            child: Text(context.l10n.introductionConditionsScreenConditionsCta),
            onPressed: () => PlaceholderScreen.show(context, secured: false),
          ),
          const SizedBox(height: 8),
          ElevatedButton(
            key: const Key('introductionConditionsScreenNextCta'),
            onPressed: () => Navigator.of(context).restorablePushNamedAndRemoveUntil(
              WalletRoutes.setupSecurityRoute,
              ModalRoute.withName(WalletRoutes.splashRoute),
            ),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Icon(Icons.arrow_forward, size: 16),
                const SizedBox(width: 8),
                Text(context.l10n.introductionConditionsScreenNextCta),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
