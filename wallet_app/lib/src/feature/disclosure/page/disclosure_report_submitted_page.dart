import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/page/flow_terminal_page.dart';

class DisclosureReportSubmittedPage extends StatelessWidget {
  final VoidCallback onClosePressed;

  const DisclosureReportSubmittedPage({
    required this.onClosePressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return FlowTerminalPage(
      icon: Icons.gpp_maybe_outlined,
      iconColor: context.theme.primaryColorDark,
      title: context.l10n.disclosureReportSubmittedPageTitle,
      description: context.l10n.disclosureReportSubmittedPageSubtitle,
      closeButtonCta: context.l10n.disclosureReportSubmittedPageCloseCta,
      onClosePressed: onClosePressed,
    );
  }
}
