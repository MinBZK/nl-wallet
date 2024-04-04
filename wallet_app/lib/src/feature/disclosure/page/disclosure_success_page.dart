import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/page/terminal_page.dart';

class DisclosureSuccessPage extends StatelessWidget {
  final LocalizedText organizationDisplayName;
  final VoidCallback? onHistoryPressed;
  final Function(String?) onPrimaryPressed;
  final String? returnUrl;
  final bool isLoginFlow;

  const DisclosureSuccessPage({
    required this.organizationDisplayName,
    required this.onPrimaryPressed,
    this.returnUrl,
    this.onHistoryPressed,
    this.isLoginFlow = false,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final title = isLoginFlow
        ? context.l10n.disclosureSuccessPageDescriptionForLogin(organizationDisplayName.l10nValue(context))
        : context.l10n.disclosureSuccessPageDescription(organizationDisplayName.l10nValue(context));
    bool hasReturnUrl = returnUrl != null;
    return TerminalPage(
      title: context.l10n.disclosureSuccessPageTitle,
      onPrimaryPressed: () => onPrimaryPressed(returnUrl),
      primaryButtonIcon: hasReturnUrl ? Icons.close_outlined : Icons.arrow_forward_outlined,
      description: title,
      primaryButtonCta: _resolvePrimaryCta(context),
      secondaryButtonCta: context.l10n.disclosureSuccessPageShowHistoryCta,
      onSecondaryButtonPressed: onHistoryPressed,
    );
  }

  String _resolvePrimaryCta(BuildContext context) {
    if (returnUrl != null) {
      if (isLoginFlow) {
        return context.l10n.disclosureSuccessPageToWebsiteCta;
      } else {
        return context.l10n.disclosureSuccessPageCloseCta;
      }
    } else {
      if (isLoginFlow) {
        return context.l10n.disclosureSuccessPageCloseCta;
      } else {
        return context.l10n.disclosureSuccessPageToDashboardCta;
      }
    }
  }
}
