import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/page_illustration.dart';
import '../../common/page/terminal_page.dart';

class IssuanceStoppedPage extends StatelessWidget {
  final VoidCallback onGiveFeedbackPressed;
  final VoidCallback onClosePressed;

  const IssuanceStoppedPage({
    required this.onClosePressed,
    required this.onGiveFeedbackPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return TerminalPage(
      illustration: const PageIllustration(asset: WalletAssets.svg_stopped),
      title: context.l10n.issuanceStoppedPageTitle,
      description: context.l10n.issuanceStoppedPageDescription,
      primaryButtonCta: context.l10n.issuanceStoppedPageCloseCta,
      onPrimaryPressed: onClosePressed,
      secondaryButtonCta: context.l10n.issuanceStoppedPageGiveFeedbackCta,
      onSecondaryButtonPressed: onGiveFeedbackPressed,
    );
  }
}
