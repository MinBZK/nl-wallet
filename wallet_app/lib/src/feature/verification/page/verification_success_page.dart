import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/page/flow_terminal_page.dart';

class VerificationSuccessPage extends StatelessWidget {
  final String verifierShortName;
  final VoidCallback? onHistoryPressed;
  final VoidCallback onClosePressed;

  const VerificationSuccessPage({
    required this.verifierShortName,
    required this.onClosePressed,
    this.onHistoryPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return FlowTerminalPage(
      icon: Icons.check,
      title: context.l10n.verificationSuccessPageTitle,
      description: context.l10n.verificationSuccessPageDescription(verifierShortName),
      closeButtonCta: context.l10n.verificationSuccessPageCloseCta,
      onClosePressed: onClosePressed,
      tertiaryButtonCta: context.l10n.verificationSuccessPageShowHistoryCta,
      onTertiaryButtonPressed: onHistoryPressed,
    );
  }
}
