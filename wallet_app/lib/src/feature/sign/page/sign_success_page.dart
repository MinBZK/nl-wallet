import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/flow_terminal_page.dart';

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
    final locale = AppLocalizations.of(context);
    return FlowTerminalPage(
      icon: Icons.check,
      title: locale.signSuccessPageTitle,
      description: locale.signSuccessPageDescription(organizationName),
      closeButtonCta: locale.signSuccessPageCloseCta,
      onClosePressed: onClosePressed,
      tertiaryButtonCta: locale.signSuccessPageHistoryCta,
      onTertiaryButtonPressed: onHistoryPressed,
    );
  }
}
