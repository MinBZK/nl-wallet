import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import '../../../domain/model/result/application_error.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/terminal_page.dart';
import '../../common/sheet/error_details_sheet.dart';
import '../../common/widget/page_illustration.dart';

class DisclosureRelyingPartyErrorPage extends StatelessWidget {
  final VoidCallback onClosePressed;
  final String? organizationName;
  final ApplicationError? error;

  const DisclosureRelyingPartyErrorPage({
    required this.onClosePressed,
    this.organizationName,
    this.error,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final description = organizationName == null
        ? context.l10n.disclosureRelyingPartyErrorDescription
        : context.l10n.disclosureRelyingPartyErrorDescriptionWithOrganizationName(organizationName!);
    return TerminalPage(
      illustration: const PageIllustration(asset: WalletAssets.svg_error_general),
      title: context.l10n.disclosureRelyingPartyErrorTitle,
      description: description,
      primaryButtonCta: context.l10n.disclosureRelyingPartyErrorCloseCta,
      onPrimaryPressed: onClosePressed,
      secondaryButtonCta: context.l10n.generalShowDetailsCta,
      secondaryButtonIcon: Icon(Icons.info_outline_rounded),
      onSecondaryButtonPressed: !kReleaseMode ? () => ErrorDetailsSheet.show(context, error: error) : null,
      flipButtonOrder: true,
    );
  }
}
