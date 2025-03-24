import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/page_illustration.dart';
import '../../common/page/terminal_page.dart';

class IssuanceIdentityValidationFailedPage extends StatelessWidget {
  final VoidCallback onSomethingNotRightPressed;
  final VoidCallback onClosePressed;

  const IssuanceIdentityValidationFailedPage({
    required this.onClosePressed,
    required this.onSomethingNotRightPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return TerminalPage(
      illustration: const PageIllustration(asset: WalletAssets.svg_error_general),
      title: context.l10n.issuanceIdentityValidationFailedPageTitle,
      description: context.l10n.issuanceIdentityValidationFailedPageDescription,
      primaryButtonCta: context.l10n.issuanceIdentityValidationFailedPageCloseCta,
      onPrimaryPressed: onClosePressed,
      secondaryButtonCta: context.l10n.issuanceIdentityValidationFailedPageSomethingNotRightCta,
      onSecondaryButtonPressed: onSomethingNotRightPressed,
    );
  }
}
