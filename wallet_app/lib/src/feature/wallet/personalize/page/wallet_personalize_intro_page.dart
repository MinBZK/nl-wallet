import 'package:flutter/material.dart';

import '../../../../domain/model/flow_progress.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../wallet_assets.dart';
import '../../../common/page/terminal_page.dart';
import '../../../common/widget/page_illustration.dart';

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
    return TerminalPage(
      title: context.l10n.walletPersonalizeIntroPageTitle,
      description: context.l10n.walletPersonalizeIntroPageDescription,
      illustration: const PageIllustration(asset: WalletAssets.svg_digid),
      primaryButtonCta: context.l10n.walletPersonalizeIntroPageLoginWithDigidCta,
      onPrimaryPressed: onDigidLoginPressed,
      primaryButtonIcon: ExcludeSemantics(child: Image.asset(WalletAssets.logo_digid)),
      secondaryButtonCta: context.l10n.walletPersonalizeIntroPageDigidWebsiteCta,
      onSecondaryButtonPressed: onDigidWebsitePressed,
      secondaryButtonIcon: const Icon(Icons.arrow_outward_rounded),
    );
  }
}
