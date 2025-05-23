import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/object_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/terminal_page.dart';
import '../../common/widget/page_illustration.dart';

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
      primaryButtonIcon: Icon(hasReturnUrl ? Icons.north_east : Icons.arrow_forward),
      description: title,
      illustration: const PageIllustration(asset: WalletAssets.svg_sharing_success),
      primaryButtonCta: hasReturnUrl
          ? context.l10n.disclosureSuccessPageToWebsiteCta
          : context.l10n.disclosureSuccessPageToDashboardCta,
      secondaryButtonCta: context.l10n.disclosureSuccessPageShowHistoryCta.takeIf((_) => onHistoryPressed != null),
      onSecondaryButtonPressed: onHistoryPressed,
    );
  }
}
