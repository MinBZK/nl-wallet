import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
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
    return FlowTerminalPage(
      icon: Icons.not_interested,
      iconColor: context.theme.primaryColorDark,
      title: context.l10n.signStoppedPageTitle,
      description: context.l10n.signStoppedPageDescription,
      closeButtonCta: context.l10n.signStoppedPageCloseCta,
      onClosePressed: onClosePressed,
      secondaryButtonCta: context.l10n.signStoppedPageFeedbackCta,
      onSecondaryButtonPressed: onGiveFeedbackPressed,
    );
  }
}
