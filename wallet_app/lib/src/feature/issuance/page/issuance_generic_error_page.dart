import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/page/legacy_terminal_page.dart';

class IssuanceGenericErrorPage extends StatelessWidget {
  final VoidCallback onClosePressed;

  const IssuanceGenericErrorPage({
    required this.onClosePressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return LegacyTerminalPage(
      icon: Icons.priority_high,
      iconColor: context.theme.primaryColorDark,
      title: context.l10n.issuanceGenericErrorPageTitle,
      description: context.l10n.issuanceGenericErrorPageDescription,
      primaryButtonCta: context.l10n.issuanceGenericErrorPageCloseCta,
      onPrimaryPressed: onClosePressed,
    );
  }
}
