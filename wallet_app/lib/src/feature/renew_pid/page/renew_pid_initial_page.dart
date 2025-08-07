import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/terminal_page.dart';
import '../../common/widget/page_illustration.dart';

class RenewPidInitialPage extends StatelessWidget {
  final VoidCallback onPrimaryPressed;
  final VoidCallback onSecondaryButtonPressed;

  const RenewPidInitialPage({
    required this.onPrimaryPressed,
    required this.onSecondaryButtonPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return TerminalPage(
      title: context.l10n.renewPidIntroPageTitle,
      description: context.l10n.renewPidIntroPageDescription,
      primaryButtonCta: context.l10n.renewPidIntroPageLoginWithDigidCta,
      onPrimaryPressed: onPrimaryPressed,
      primaryButtonIcon: Image.asset(WalletAssets.logo_digid),
      secondaryButtonCta: context.l10n.renewPidIntroPageDigidWebsiteCta,
      secondaryButtonIcon: const Icon(Icons.north_east_outlined),
      onSecondaryButtonPressed: onSecondaryButtonPressed,
      illustration: const PageIllustration(asset: WalletAssets.svg_digid),
    );
  }
}
