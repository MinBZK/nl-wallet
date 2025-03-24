import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/page_illustration.dart';
import '../../common/page/terminal_page.dart';

class SignGenericErrorPage extends StatelessWidget {
  final VoidCallback onClosePressed;

  const SignGenericErrorPage({
    required this.onClosePressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return TerminalPage(
      illustration: const PageIllustration(asset: WalletAssets.svg_error_general),
      title: context.l10n.signGenericErrorPageTitle,
      description: context.l10n.signGenericErrorPageDescription,
      primaryButtonCta: context.l10n.signGenericErrorPageCloseCta,
      onPrimaryPressed: onClosePressed,
    );
  }
}
