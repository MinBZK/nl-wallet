import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/flow_terminal_page.dart';

class SignStoppedPage extends StatelessWidget {
  final VoidCallback? onGiveFeedbackPressed;
  final VoidCallback onClosePressed;

  const SignStoppedPage({
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
      title: locale.signStoppedPageTitle,
      description: locale.signStoppedPageDescription,
      closeButtonCta: locale.signStoppedPageCloseCta,
      onClosePressed: onClosePressed,
      secondaryButtonCta: locale.signStoppedPageFeedbackCta,
      onSecondaryButtonPressed: onGiveFeedbackPressed,
    );
  }
}
