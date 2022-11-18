import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import 'issuance_terminal_page.dart';

class IssuanceGenericErrorPage extends StatelessWidget {
  final VoidCallback onClosePressed;

  const IssuanceGenericErrorPage({
    required this.onClosePressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return IssuanceTerminalPage(
      icon: Icons.priority_high,
      title: locale.issuanceGenericErrorPageTitle,
      description: locale.issuanceGenericErrorPageDescription,
      closeButtonCta: locale.issuanceGenericErrorPageCloseCta,
      onClosePressed: onClosePressed,
    );
  }
}
