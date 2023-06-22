import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/flow_terminal_page.dart';

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
    return FlowTerminalPage(
      icon: Icons.not_interested,
      iconColor: context.theme.primaryColorDark,
      title: context.l10n.issuanceStoppedPageTitle,
      description: context.l10n.issuanceStoppedPageDescription,
      closeButtonCta: context.l10n.issuanceStoppedPageCloseCta,
      onClosePressed: onClosePressed,
      secondaryButtonCta: context.l10n.issuanceStoppedPageGiveFeedbackCta,
      onSecondaryButtonPressed: onGiveFeedbackPressed,
    );
  }
}
