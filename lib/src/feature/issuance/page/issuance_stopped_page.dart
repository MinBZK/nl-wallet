import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import 'issuance_terminal_page.dart';

class IssuanceStoppedPage extends StatelessWidget {
  final VoidCallback onGiveFeedbackPressed;
  final VoidCallback onClosePressed;

  const IssuanceStoppedPage({
    required this.onClosePressed,
    required this.onGiveFeedbackPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return IssuanceTerminalPage(
      icon: Icons.not_interested,
      title: locale.issuanceStoppedPageTitle,
      description: locale.issuanceStoppedPageDescription,
      closeButtonCta: locale.issuanceStoppedPageCloseCta,
      onClosePressed: onClosePressed,
      secondaryButtonCta: locale.issuanceStoppedPageGiveFeedbackCta,
      onSecondaryButtonPressed: onGiveFeedbackPressed,
    );
  }
}
