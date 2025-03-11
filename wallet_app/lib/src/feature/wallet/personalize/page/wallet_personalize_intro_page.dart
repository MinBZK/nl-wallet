import 'package:flutter/material.dart';

import '../../../../domain/model/flow_progress.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';
import '../../../../wallet_assets.dart';
import '../../../common/page/page_illustration.dart';
import '../../../common/widget/button/confirm/confirm_buttons.dart';
import '../../../common/widget/button/primary_button.dart';
import '../../../common/widget/button/tertiary_button.dart';
import '../../../common/widget/sliver_wallet_app_bar.dart';
import '../../../common/widget/text/body_text.dart';
import '../../../common/widget/wallet_scrollbar.dart';

class WalletPersonalizeIntroPage extends StatelessWidget {
  final VoidCallback onDigidLoginPressed;
  final VoidCallback onDigidWebsitePressed;
  final FlowProgress? progress;

  const WalletPersonalizeIntroPage({
    required this.onDigidLoginPressed,
    required this.onDigidWebsitePressed,
    this.progress,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Column(
        children: [
          Expanded(
            child: WalletScrollbar(
              child: CustomScrollView(
                slivers: [
                  SliverWalletAppBar(
                    title: context.l10n.walletPersonalizeIntroPageTitle,
                    progress: progress,
                    scrollController: PrimaryScrollController.maybeOf(context),
                  ),
                  SliverPadding(
                    padding: const EdgeInsets.symmetric(horizontal: 16),
                    sliver: SliverList(
                      delegate: SliverChildListDelegate([
                        BodyText(context.l10n.walletPersonalizeIntroPageDescription),
                        const SizedBox(height: 32),
                        const PageIllustration(
                          asset: WalletAssets.svg_digid,
                          padding: EdgeInsets.zero,
                        ),
                        const SizedBox(height: 32),
                      ]),
                    ),
                  ),
                ],
              ),
            ),
          ),
          _buildBottomSection(context),
        ],
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        const Divider(),
        ConfirmButtons(
          primaryButton: PrimaryButton(
            key: const Key('digidLoginCta'),
            onPressed: onDigidLoginPressed,
            text: Text.rich(context.l10n.walletPersonalizeIntroPageLoginWithDigidCta.toTextSpan(context)),
            icon: ExcludeSemantics(child: Image.asset(WalletAssets.logo_digid)),
          ),
          secondaryButton: TertiaryButton(
            key: const Key('digidWebsiteCta'),
            onPressed: onDigidWebsitePressed,
            icon: const Icon(Icons.arrow_outward_rounded),
            text: Text.rich(context.l10n.walletPersonalizeIntroPageDigidWebsiteCta.toTextSpan(context)),
          ),
        ),
      ],
    );
  }
}
