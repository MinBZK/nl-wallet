import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/page/flow_terminal_page.dart';

class SignSuccessPage extends StatelessWidget {
  final String organizationName;
  final VoidCallback? onHistoryPressed;
  final VoidCallback onClosePressed;

  const SignSuccessPage({
    required this.organizationName,
    required this.onClosePressed,
    this.onHistoryPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return FlowTerminalPage(
      icon: Icons.check,
      title: context.l10n.signSuccessPageTitle,
      description: context.l10n.signSuccessPageDescription(organizationName),
      closeButtonCta: context.l10n.signSuccessPageCloseCta,
      onClosePressed: onClosePressed,
      tertiaryButtonCta: context.l10n.signSuccessPageHistoryCta,
      onTertiaryButtonPressed: onHistoryPressed,
    );
  }
}
