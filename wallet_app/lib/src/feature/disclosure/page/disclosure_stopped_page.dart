import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/organization.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/terminal_page.dart';
import '../../common/widget/page_illustration.dart';

class DisclosureStoppedPage extends StatelessWidget {
  final Organization organization;
  final Function(String?) onClosePressed;
  final bool isLoginFlow;
  final String? returnUrl;

  const DisclosureStoppedPage({
    required this.onClosePressed,
    required this.organization,
    this.isLoginFlow = false,
    this.returnUrl,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final description = isLoginFlow
        ? context.l10n.disclosureStoppedPageDescriptionForLogin(organization.displayName.l10nValue(context))
        : context.l10n.disclosureStoppedPageDescription(organization.displayName.l10nValue(context));
    final bool hasReturnUrl = returnUrl != null;
    return TerminalPage(
      title: context.l10n.disclosureStoppedPageTitle,
      description: description,
      illustration: const PageIllustration(asset: WalletAssets.svg_stopped),
      primaryButtonCta:
          hasReturnUrl ? context.l10n.disclosureStoppedPageToWebsiteCta : context.l10n.disclosureStoppedPageCloseCta,
      primaryButtonIcon: Icon(hasReturnUrl ? Icons.north_east : Icons.close_outlined),
      onPrimaryPressed: () => onClosePressed(returnUrl),
    );
  }
}
