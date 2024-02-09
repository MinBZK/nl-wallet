import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/page/legacy_terminal_page.dart';

class SignGenericErrorPage extends StatelessWidget {
  final VoidCallback onClosePressed;

  const SignGenericErrorPage({
    required this.onClosePressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return LegacyTerminalPage(
      icon: Icons.not_interested,
      iconColor: context.theme.primaryColorDark,
      title: context.l10n.signGenericErrorPageTitle,
      description: context.l10n.signGenericErrorPageDescription,
      primaryButtonCta: context.l10n.signGenericErrorPageCloseCta,
      onPrimaryPressed: onClosePressed,
    );
  }
}
