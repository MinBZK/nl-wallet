import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/page/flow_terminal_page.dart';

class DisclosureGenericErrorPage extends StatelessWidget {
  final VoidCallback onClosePressed;

  const DisclosureGenericErrorPage({
    required this.onClosePressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return FlowTerminalPage(
      icon: Icons.not_interested,
      iconColor: context.theme.primaryColorDark,
      title: context.l10n.verificationGenericErrorPageTitle,
      description: context.l10n.verificationGenericErrorPageDescription,
      closeButtonCta: context.l10n.verificationGenericErrorPageCloseCta,
      onClosePressed: onClosePressed,
    );
  }
}
