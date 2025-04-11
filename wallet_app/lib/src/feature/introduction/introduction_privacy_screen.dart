import 'package:flutter/material.dart';

import '../../domain/model/flow_progress.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../wallet_assets.dart';
import '../../wallet_constants.dart';
import '../common/page/page_illustration.dart';
import '../common/widget/bullet_list.dart';
import '../common/widget/button/confirm/confirm_buttons.dart';
import '../common/widget/button/icon/back_icon_button.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/button/primary_button.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/sliver_sized_box.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';

class IntroductionPrivacyScreen extends StatelessWidget {
  const IntroductionPrivacyScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('introductionPrivacyScreen'),
      body: SafeArea(child: _buildContent(context)),
    );
  }

  Widget _buildContent(BuildContext context) {
    return Column(
      children: [
        Expanded(
          child: WalletScrollbar(
            child: CustomScrollView(
              slivers: [
                SliverWalletAppBar(
                  title: context.l10n.introductionPrivacyScreenHeadline,
                  scrollController: PrimaryScrollController.maybeOf(context),
                  leading: const BackIconButton(),
                  progress: const FlowProgress(currentStep: 1, totalSteps: kSetupSteps),
                  actions: const [HelpIconButton()],
                ),
                SliverPadding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  sliver: SliverToBoxAdapter(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        BulletList(
                          items: context.l10n.introductionPrivacyScreenBulletPoints.split('\n'),
                          icon: Icon(
                            Icons.check,
                            color: context.colorScheme.primary,
                            size: 18,
                          ),
                          rowPadding: EdgeInsets.symmetric(vertical: 4),
                        ),
                      ],
                    ),
                  ),
                ),
                const SliverToBoxAdapter(
                  child: PageIllustration(asset: WalletAssets.svg_privacy),
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
    final FitsWidthWidget nextButton = PrimaryButton(
      key: const Key('introductionPrivacyScreenNextCta'),
      onPressed: () => Navigator.of(context).restorablePushNamed(WalletRoutes.setupSecurityRoute),
      text: Text.rich(context.l10n.introductionPrivacyScreenNextCta.toTextSpan(context)),
      icon: const Icon(Icons.arrow_forward_rounded),
    );
    final FitsWidthWidget privacyButton = TertiaryButton(
      key: const Key('introductionPrivacyScreenPrivacyCta'),
      onPressed: () => Navigator.pushNamed(context, WalletRoutes.privacyPolicyRoute),
      text: Text.rich(context.l10n.introductionPrivacyScreenPrivacyCta.toTextSpan(context)),
      icon: const Icon(Icons.arrow_forward_rounded),
    );
    return Column(
      mainAxisSize: MainAxisSize.min,
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        const Divider(),
        ConfirmButtons(
          secondaryButton: context.isLandscape ? privacyButton : nextButton,
          primaryButton: context.isLandscape ? nextButton : privacyButton,
        ),
      ],
    );
  }
}
