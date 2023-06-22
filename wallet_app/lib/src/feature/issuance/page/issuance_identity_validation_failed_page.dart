import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/flow_terminal_page.dart';

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
    return FlowTerminalPage(
      icon: Icons.priority_high,
      iconColor: context.theme.primaryColorDark,
      title: context.l10n.issuanceIdentityValidationFailedPageTitle,
      description: context.l10n.issuanceIdentityValidationFailedPageDescription,
      closeButtonCta: context.l10n.issuanceIdentityValidationFailedPageCloseCta,
      onClosePressed: onClosePressed,
      secondaryButtonCta: context.l10n.issuanceIdentityValidationFailedPageSomethingNotRightCta,
      onSecondaryButtonPressed: onSomethingNotRightPressed,
    );
  }
}
