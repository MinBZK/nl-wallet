import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/flow_terminal_page.dart';

class VerificationSuccessPage extends StatelessWidget {
  final String verifierShortName;
  final VoidCallback? onHistoryPressed;
  final VoidCallback onClosePressed;

  const VerificationSuccessPage({
    required this.verifierShortName,
    required this.onClosePressed,
    this.onHistoryPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return FlowTerminalPage(
      icon: Icons.check,
      title: locale.verificationSuccessPageTitle(verifierShortName),
      description: locale.verificationSuccessPageHistoryDescription,
      closeButtonCta: locale.verificationSuccessPageCloseCta,
      onClosePressed: onClosePressed,
      tertiaryButtonCta: locale.verificationSuccessPageShowHistoryCta,
      onTertiaryButtonPressed: onHistoryPressed,
    );
  }
}
