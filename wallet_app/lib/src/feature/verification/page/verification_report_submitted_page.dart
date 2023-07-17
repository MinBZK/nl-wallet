import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/page/flow_terminal_page.dart';

class VerificationReportSubmittedPage extends StatelessWidget {
  final VoidCallback onClosePressed;

  const VerificationReportSubmittedPage({
    required this.onClosePressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return FlowTerminalPage(
      icon: Icons.gpp_maybe_outlined,
      iconColor: context.theme.primaryColorDark,
      title: context.l10n.verificationReportSubmittedPageTitle,
      description: context.l10n.verificationReportSubmittedPageSubtitle,
      closeButtonCta: context.l10n.verificationReportSubmittedPageCloseCta,
      onClosePressed: onClosePressed,
    );
  }
}
