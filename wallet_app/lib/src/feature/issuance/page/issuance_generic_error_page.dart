import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/terminal_page.dart';
import '../../common/widget/page_illustration.dart';

class IssuanceGenericErrorPage extends StatelessWidget {
  final VoidCallback onClosePressed;

  const IssuanceGenericErrorPage({
    required this.onClosePressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return TerminalPage(
      illustration: const PageIllustration(asset: WalletAssets.svg_error_general),
      title: context.l10n.issuanceGenericErrorPageTitle,
      description: context.l10n.issuanceGenericErrorPageDescription,
      primaryButtonCta: context.l10n.issuanceGenericErrorPageCloseCta,
      onPrimaryPressed: onClosePressed,
    );
  }
}
