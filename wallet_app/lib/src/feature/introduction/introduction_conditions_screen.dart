import 'package:flutter/material.dart';

import '../../domain/model/flow_progress.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../../wallet_constants.dart';
import '../common/page/page_illustration.dart';
import '../common/screen/placeholder_screen.dart';
import '../common/widget/bullet_list.dart';
import '../common/widget/button/confirm/confirm_buttons.dart';
import '../common/widget/button/icon/back_icon_button.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/button/primary_button.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/sliver_sized_box.dart';
import '../common/widget/sliver_wallet_app_bar.dart';

class IntroductionConditionsScreen extends StatelessWidget {
  const IntroductionConditionsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('introductionConditionsScreen'),
      body: SafeArea(child: _buildContent(context)),
    );
  }

  Widget _buildContent(BuildContext context) {
    return Column(
      children: [
        Expanded(
          child: Scrollbar(
            thumbVisibility: true,
            child: CustomScrollView(
              slivers: [
                SliverWalletAppBar(
                  title: context.l10n.introductionConditionsScreenHeadline,
                  progress: const FlowProgress(currentStep: 2, totalSteps: kSetupSteps),
                  leading: const BackIconButton(),
                  actions: [
                    HelpIconButton(
                      onPressed: () => PlaceholderScreen.show(context, secured: false),
                    )
                  ],
                ),
                SliverPadding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  sliver: SliverToBoxAdapter(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        BulletList(
                          items: context.l10n.introductionConditionsScreenBulletPoints.split('\n'),
                        ),
                      ],
                    ),
                  ),
                ),
                const SliverToBoxAdapter(
                  child: PageIllustration(asset: WalletAssets.svg_terms),
                ),
                const SliverSizedBox(height: 24),
              ],
            ),
          ),
        ),
        _buildBottomSection(context),
      ],
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    FitsWidthWidget nextButton = PrimaryButton(
      key: const Key('introductionConditionsScreenNextCta'),
      onPressed: () => Navigator.of(context).restorablePushNamed(WalletRoutes.setupSecurityRoute),
      icon: const Icon(Icons.arrow_forward_rounded),
      text: Text(context.l10n.introductionConditionsScreenNextCta),
    );
    FitsWidthWidget conditionsButton = TertiaryButton(
      text: Text(context.l10n.introductionConditionsScreenConditionsCta),
      icon: const Icon(Icons.arrow_forward_rounded),
      onPressed: () => PlaceholderScreen.show(context, secured: false),
      key: const Key('introductionConditionsScreenConditionsCta'),
    );
    return Column(
      mainAxisSize: MainAxisSize.min,
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        const Divider(height: 1),
        ConfirmButtons(
          secondaryButton: context.isLandscape ? conditionsButton : nextButton,
          primaryButton: context.isLandscape ? nextButton : conditionsButton,
        ),
      ],
    );
  }
}
