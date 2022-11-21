import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/flow_terminal_page.dart';

class VerificationGenericErrorPage extends StatelessWidget {
  final VoidCallback onClosePressed;

  const VerificationGenericErrorPage({
    required this.onClosePressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return FlowTerminalPage(
      icon: Icons.not_interested,
      iconColor: Theme.of(context).primaryColorDark,
      title: locale.verificationGenericErrorPageTitle,
      description: locale.verificationGenericErrorPageDescription,
      closeButtonCta: locale.verificationGenericErrorPageCloseCta,
      onClosePressed: onClosePressed,
    );
  }
}
