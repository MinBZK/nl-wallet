import 'package:flutter/material.dart';

import '../../../../domain/model/flow_progress.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../wallet_assets.dart';
import '../../../common/page/terminal_page.dart';
import '../../../common/widget/button/primary_button.dart';
import '../../../common/widget/button/tertiary_button.dart';
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
      primaryButton: PrimaryButton(
        text: Text(context.l10n.walletPersonalizeIntroPageLoginWithDigidCta),
        icon: ExcludeSemantics(child: Image.asset(WalletAssets.logo_digid)),
        onPressed: onDigidLoginPressed,
        key: const Key('primaryButtonCta'),
      ),
      secondaryButton: TertiaryButton(
        text: Text(context.l10n.walletPersonalizeIntroPageDigidWebsiteCta),
        icon: const Icon(Icons.arrow_outward_rounded),
        onPressed: onDigidWebsitePressed,
        key: const Key('secondaryButtonCta'),
      ),
    );
  }
}
