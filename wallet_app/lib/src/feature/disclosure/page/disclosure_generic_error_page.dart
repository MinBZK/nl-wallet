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
      title: context.l10n.disclosureGenericErrorPageTitle,
      description: context.l10n.disclosureGenericErrorPageDescription,
      primaryButtonCta: context.l10n.disclosureGenericErrorPageCloseCta,
      onPrimaryPressed: onClosePressed,
    );
  }
}
