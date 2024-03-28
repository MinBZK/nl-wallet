import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/page/terminal_page.dart';

class DisclosureSuccessPage extends StatelessWidget {
  final LocalizedText organizationDisplayName;
  final VoidCallback? onHistoryPressed;
  final Function(String?) onPrimaryPressed;
  final String? returnUrl;

  const DisclosureSuccessPage({
    required this.organizationDisplayName,
    required this.onPrimaryPressed,
    this.returnUrl,
    this.onHistoryPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    bool hasReturnUrl = returnUrl != null;
    return TerminalPage(
      title: context.l10n.disclosureSuccessPageTitle,
      description: context.l10n.disclosureSuccessPageDescription(organizationDisplayName.l10nValue(context)),
      primaryButtonCta:
          hasReturnUrl ? context.l10n.disclosureSuccessPageCloseCta : context.l10n.disclosureSuccessPageToDashboardCta,
      onPrimaryPressed: () => onPrimaryPressed(returnUrl),
      primaryButtonIcon: hasReturnUrl ? Icons.close_outlined : Icons.arrow_forward_outlined,
      secondaryButtonCta: context.l10n.disclosureSuccessPageShowHistoryCta,
      onSecondaryButtonPressed: onHistoryPressed,
    );
  }
}
