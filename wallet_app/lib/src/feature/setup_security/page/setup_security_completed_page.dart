import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/page_illustration.dart';
import '../../common/page/terminal_page.dart';

class SetupSecurityCompletedPage extends StatelessWidget {
  final VoidCallback onSetupWalletPressed;

  const SetupSecurityCompletedPage({required this.onSetupWalletPressed, super.key});

  @override
  Widget build(BuildContext context) {
    return TerminalPage(
      title: context.l10n.setupSecurityCompletedPageTitle,
      primaryButtonCta: context.l10n.setupSecurityCompletedPageCreateWalletCta,
      description: context.l10n.setupSecurityCompletedPageDescription,
      onPrimaryPressed: onSetupWalletPressed,
      illustration: const PageIllustration(asset: WalletAssets.svg_pin_set),
    );
  }
}
