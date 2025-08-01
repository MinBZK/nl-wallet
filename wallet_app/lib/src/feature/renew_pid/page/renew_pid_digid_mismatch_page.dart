import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/terminal_page.dart';
import '../../common/widget/page_illustration.dart';

class RenewPidDigidMismatchPage extends StatelessWidget {
  final VoidCallback onPrimaryPressed;
  final VoidCallback onSecondaryButtonPressed;

  const RenewPidDigidMismatchPage({
    required this.onPrimaryPressed,
    required this.onSecondaryButtonPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return TerminalPage(
      title: context.l10n.renewPidDigidMismatchPageTitle,
      description: context.l10n.renewPidDigidMismatchPageDescription,
      illustration: const PageIllustration(asset: WalletAssets.svg_stopped),
      primaryButtonCta: context.l10n.renewPidDigidMismatchPageRetryCta,
      primaryButtonIcon: Image.asset(WalletAssets.logo_digid),
      onPrimaryPressed: onPrimaryPressed,
      onSecondaryButtonPressed: onSecondaryButtonPressed,
      secondaryButtonIcon: const Icon(Icons.north_east_outlined),
      secondaryButtonCta: context.l10n.renewPidDigidMismatchPageOpenWebsiteCta,
    );
  }
}
