import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/disclosure/return_url_case.dart';
import '../../../domain/model/organization.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/page_illustration.dart';
import '../../common/page/terminal_page.dart';

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
    return TerminalPage(
      title: context.l10n.disclosureStoppedPageTitle,
      description: description,
      illustration: const PageIllustration(asset: WalletAssets.svg_stopped),
      primaryButtonCta: _resolvePrimaryCta(context),
      primaryButtonIcon: Icon(returnUrl == null ? Icons.close_outlined : Icons.arrow_forward_outlined),
      onPrimaryPressed: () => onClosePressed(returnUrl),
    );
  }

  String _resolvePrimaryCta(BuildContext context) {
    final returnUrlCase = ReturnUrlCase.resolve(isLoginFlow: isLoginFlow, hasReturnUrl: returnUrl != null);
    return switch (returnUrlCase) {
      ReturnUrlCase.returnUrl => context.l10n.disclosureStoppedPageCloseCta,
      ReturnUrlCase.noReturnUrl => context.l10n.disclosureStoppedPageCloseCta,
      ReturnUrlCase.loginReturnUrl => context.l10n.disclosureStoppedPageToWebsiteCta,
      ReturnUrlCase.loginNoReturnUrl => context.l10n.disclosureStoppedPageCloseCta,
    };
  }
}
