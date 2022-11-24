import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/flow_terminal_page.dart';

class SetupSecurityCompletedPage extends StatelessWidget {
  final VoidCallback onSetupWalletPressed;

  const SetupSecurityCompletedPage({required this.onSetupWalletPressed, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return FlowTerminalPage(
      icon: Icons.check,
      title: locale.setupSecurityCompletedPageTitle,
      closeButtonCta: locale.setupSecurityCompletedPageCreateWalletCta,
      description: locale.setupSecurityCompletedPageDescription,
      onClosePressed: onSetupWalletPressed,
    );
  }
}
