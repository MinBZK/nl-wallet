import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/flow_terminal_page.dart';

class VerificationStoppedPage extends StatelessWidget {
  final VoidCallback? onGiveFeedbackPressed;
  final VoidCallback onClosePressed;

  const VerificationStoppedPage({
    required this.onClosePressed,
    this.onGiveFeedbackPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return FlowTerminalPage(
      icon: Icons.not_interested,
      iconColor: Theme.of(context).primaryColorDark,
      title: locale.verificationDeclinedPageTitle,
      description: locale.verificationDeclinedPageDescription,
      closeButtonCta: locale.verificationDeclinedPageCloseCta,
      onClosePressed: onClosePressed,
      secondaryButtonCta: locale.verificationDeclinedPageFeedbackCta,
      onSecondaryButtonPressed: onGiveFeedbackPressed,
    );
  }
}
