import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/page/legacy_terminal_page.dart';

class DisclosureReportSubmittedPage extends StatelessWidget {
  final VoidCallback onClosePressed;

  const DisclosureReportSubmittedPage({
    required this.onClosePressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return LegacyTerminalPage(
      icon: Icons.gpp_maybe_outlined,
      iconColor: context.theme.primaryColorDark,
      title: context.l10n.disclosureReportSubmittedPageTitle,
      description: context.l10n.disclosureReportSubmittedPageSubtitle,
      primaryButtonCta: context.l10n.disclosureReportSubmittedPageCloseCta,
      onPrimaryPressed: onClosePressed,
    );
  }
}
