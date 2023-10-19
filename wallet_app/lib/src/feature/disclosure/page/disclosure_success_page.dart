import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/page/flow_terminal_page.dart';

class DisclosureSuccessPage extends StatelessWidget {
  final String verifierShortName;
  final VoidCallback? onHistoryPressed;
  final VoidCallback onClosePressed;

  const DisclosureSuccessPage({
    required this.verifierShortName,
    required this.onClosePressed,
    this.onHistoryPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return FlowTerminalPage(
      icon: Icons.check,
      title: context.l10n.disclosureSuccessPageTitle,
      description: context.l10n.disclosureSuccessPageDescription(verifierShortName),
      closeButtonCta: context.l10n.disclosureSuccessPageCloseCta,
      onClosePressed: onClosePressed,
      tertiaryButtonCta: context.l10n.disclosureSuccessPageShowHistoryCta,
      onTertiaryButtonPressed: onHistoryPressed,
    );
  }
}
