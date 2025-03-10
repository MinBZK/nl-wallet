import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/disclosure/return_url_case.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/object_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/page_illustration.dart';
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
    final bool hasReturnUrl = returnUrl != null;
    return TerminalPage(
      title: context.l10n.disclosureSuccessPageTitle,
      onPrimaryPressed: () => onPrimaryPressed(returnUrl),
      primaryButtonIcon: hasReturnUrl ? Icons.arrow_forward_outlined : Icons.close_outlined,
      description: title,
      illustration: const PageIllustration(asset: WalletAssets.svg_sharing_success),
      primaryButtonCta: _resolvePrimaryCta(context),
      secondaryButtonCta: context.l10n.disclosureSuccessPageShowHistoryCta.takeIf((_) => onHistoryPressed != null),
      onSecondaryButtonPressed: onHistoryPressed,
    );
  }

  String _resolvePrimaryCta(BuildContext context) {
    final returnUrlCase = ReturnUrlCase.resolve(isLoginFlow: isLoginFlow, hasReturnUrl: returnUrl != null);
    return switch (returnUrlCase) {
      ReturnUrlCase.returnUrl => context.l10n.disclosureSuccessPageCloseCta,
      ReturnUrlCase.noReturnUrl => context.l10n.disclosureSuccessPageToDashboardCta,
      ReturnUrlCase.loginReturnUrl => context.l10n.disclosureSuccessPageToWebsiteCta,
      ReturnUrlCase.loginNoReturnUrl => context.l10n.disclosureSuccessPageCloseCta,
    };
  }
}
