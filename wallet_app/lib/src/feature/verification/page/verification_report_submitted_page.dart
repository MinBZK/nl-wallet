import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/flow_terminal_page.dart';

class VerificationReportSubmittedPage extends StatelessWidget {
  final VoidCallback onClosePressed;

  const VerificationReportSubmittedPage({
    required this.onClosePressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return FlowTerminalPage(
      icon: Icons.gpp_maybe_outlined,
      iconColor: Theme.of(context).primaryColorDark,
      title: locale.verificationReportSubmittedPageTitle,
      description: locale.verificationReportSubmittedPageSubtitle,
      closeButtonCta: locale.verificationReportSubmittedPageCloseCta,
      onClosePressed: onClosePressed,
    );
  }
}
