import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/organization.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../sheet/missing_attributes_sheet.dart';
import '../widget/page_illustration.dart';
import 'terminal_page.dart';

class MissingAttributesPage extends StatelessWidget {
  final Organization organization;
  final List<MissingAttribute> missingAttributes;
  final VoidCallback onClosePressed;
  final bool hasReturnUrl;

  const MissingAttributesPage({
    required this.organization,
    required this.missingAttributes,
    required this.onClosePressed,
    this.hasReturnUrl = false,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final closeCta =
        hasReturnUrl ? context.l10n.missingAttributesPageBackToWebsiteCta : context.l10n.missingAttributesPageCloseCta;
    final closeIcon = hasReturnUrl ? Icons.north_east_outlined : Icons.close;
    return TerminalPage(
      title: context.l10n.missingAttributesPageTitle,
      description: context.l10n.missingAttributesPageDescription(organization.displayName.l10nValue(context)),
      primaryButtonCta: closeCta,
      onPrimaryPressed: onClosePressed,
      primaryButtonIcon: Icon(closeIcon),
      secondaryButtonCta: context.l10n.missingAttributesPageShowDetailsCta,
      onSecondaryButtonPressed: () => MissingAttributesSheet.show(context, missingAttributes),
      secondaryButtonIcon: const Icon(Icons.info_outline_rounded),
      flipButtonOrder: true,
      illustration: const PageIllustration(asset: WalletAssets.svg_error_card_blocked),
    );
  }
}
