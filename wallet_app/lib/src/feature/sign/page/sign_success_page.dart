import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/page/legacy_terminal_page.dart';

class SignSuccessPage extends StatelessWidget {
  final LocalizedText organizationName;
  final VoidCallback? onHistoryPressed;
  final VoidCallback onClosePressed;

  const SignSuccessPage({
    required this.organizationName,
    required this.onClosePressed,
    this.onHistoryPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return LegacyTerminalPage(
      icon: Icons.check,
      title: context.l10n.signSuccessPageTitle,
      description: context.l10n.signSuccessPageDescription(organizationName.l10nValue(context)),
      primaryButtonCta: context.l10n.signSuccessPageCloseCta,
      onPrimaryPressed: onClosePressed,
      tertiaryButtonCta: context.l10n.signSuccessPageHistoryCta,
      onTertiaryButtonPressed: onHistoryPressed,
    );
  }
}
