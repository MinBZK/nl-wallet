import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/page/legacy_terminal_page.dart';

class SetupSecurityCompletedPage extends StatelessWidget {
  final VoidCallback onSetupWalletPressed;

  const SetupSecurityCompletedPage({required this.onSetupWalletPressed, super.key});

  @override
  Widget build(BuildContext context) {
    return LegacyTerminalPage(
      icon: Icons.check,
      title: context.l10n.setupSecurityCompletedPageTitle,
      primaryButtonCta: context.l10n.setupSecurityCompletedPageCreateWalletCta,
      description: context.l10n.setupSecurityCompletedPageDescription,
      onPrimaryPressed: onSetupWalletPressed,
    );
  }
}
