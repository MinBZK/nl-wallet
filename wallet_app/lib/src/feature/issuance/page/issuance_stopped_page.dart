import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/terminal_page.dart';
import '../../common/widget/page_illustration.dart';

class IssuanceStoppedPage extends StatelessWidget {
  final VoidCallback onGiveFeedbackPressed;
  final Function(String?) onClosePressed;
  final String? returnUrl;

  const IssuanceStoppedPage({
    required this.onClosePressed,
    required this.onGiveFeedbackPressed,
    this.returnUrl,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final bool hasReturnUrl = returnUrl != null;
    return TerminalPage(
      illustration: const PageIllustration(asset: WalletAssets.svg_stopped),
      title: context.l10n.issuanceStoppedPageTitle,
      description: context.l10n.issuanceStoppedPageDescription,
      onPrimaryPressed: () => onClosePressed(returnUrl),
      secondaryButtonCta: context.l10n.issuanceStoppedPageGiveFeedbackCta,
      onSecondaryButtonPressed: onGiveFeedbackPressed,
      primaryButtonCta:
          hasReturnUrl ? context.l10n.issuanceStoppedPageToWebsiteCta : context.l10n.issuanceStoppedPageCloseCta,
      primaryButtonIcon: Icon(hasReturnUrl ? Icons.north_east : Icons.close_outlined),
    );
  }
}
