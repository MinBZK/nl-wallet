import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/flow_terminal_page.dart';

class IssuanceGenericErrorPage extends StatelessWidget {
  final VoidCallback onClosePressed;

  const IssuanceGenericErrorPage({
    required this.onClosePressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return FlowTerminalPage(
      icon: Icons.priority_high,
      iconColor: context.theme.primaryColorDark,
      title: context.l10n.issuanceGenericErrorPageTitle,
      description: context.l10n.issuanceGenericErrorPageDescription,
      closeButtonCta: context.l10n.issuanceGenericErrorPageCloseCta,
      onClosePressed: onClosePressed,
    );
  }
}
