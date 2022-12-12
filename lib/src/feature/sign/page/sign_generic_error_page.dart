import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/flow_terminal_page.dart';

class SignGenericErrorPage extends StatelessWidget {
  final VoidCallback onClosePressed;

  const SignGenericErrorPage({
    required this.onClosePressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return FlowTerminalPage(
      icon: Icons.not_interested,
      iconColor: Theme.of(context).primaryColorDark,
      title: locale.signGenericErrorPageTitle,
      description: locale.signGenericErrorPageDescription,
      closeButtonCta: locale.signGenericErrorPageCloseCta,
      onClosePressed: onClosePressed,
    );
  }
}
