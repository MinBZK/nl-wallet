import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import 'issuance_terminal_page.dart';

class IssuanceIdentityValidationFailedPage extends StatelessWidget {
  final VoidCallback onSomethingNotRightPressed;
  final VoidCallback onClosePressed;

  const IssuanceIdentityValidationFailedPage({
    required this.onClosePressed,
    required this.onSomethingNotRightPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return IssuanceTerminalPage(
      icon: Icons.priority_high,
      title: locale.issuanceIdentityValidationFailedPageTitle,
      description: locale.issuanceIdentityValidationFailedPageDescription,
      closeButtonCta: locale.issuanceIdentityValidationFailedPageCloseCta,
      onClosePressed: onClosePressed,
      secondaryButtonCta: locale.issuanceIdentityValidationFailedPageSomethingNotRightCta,
      onSecondaryButtonPressed: onSomethingNotRightPressed,
    );
  }
}
