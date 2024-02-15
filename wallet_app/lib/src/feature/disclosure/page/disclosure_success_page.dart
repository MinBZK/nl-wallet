import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/page/terminal_page.dart';

class DisclosureSuccessPage extends StatelessWidget {
  final LocalizedText organizationDisplayName;
  final VoidCallback? onHistoryPressed;
  final VoidCallback onClosePressed;

  const DisclosureSuccessPage({
    required this.organizationDisplayName,
    required this.onClosePressed,
    this.onHistoryPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return TerminalPage(
      title: context.l10n.disclosureSuccessPageTitle,
      description: context.l10n.disclosureSuccessPageDescription(organizationDisplayName.l10nValue(context)),
      primaryButtonCta: context.l10n.disclosureSuccessPageCloseCta,
      onPrimaryPressed: onClosePressed,
      secondaryButtonCta: context.l10n.disclosureSuccessPageShowHistoryCta,
      onSecondaryButtonPressed: onHistoryPressed,
    );
  }
}
