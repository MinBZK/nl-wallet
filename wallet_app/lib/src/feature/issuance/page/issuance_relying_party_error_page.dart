import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import '../../../domain/model/result/application_error.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/terminal_page.dart';
import '../../common/sheet/error_details_sheet.dart';
import '../../common/widget/page_illustration.dart';

class IssuanceRelyingPartyErrorPage extends StatelessWidget {
  final VoidCallback onClosePressed;
  final String? organizationName;
  final ApplicationError? error;

  const IssuanceRelyingPartyErrorPage({
    required this.onClosePressed,
    this.organizationName,
    this.error,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final description = organizationName == null
        ? context.l10n.issuanceRelyingPartyErrorDescription
        : context.l10n.issuanceRelyingPartyErrorDescriptionWithOrganizationName(organizationName!);
    return TerminalPage(
      illustration: const PageIllustration(asset: WalletAssets.svg_error_card_blocked),
      title: context.l10n.issuanceRelyingPartyErrorTitle,
      description: description,
      primaryButtonCta: context.l10n.issuanceRelyingPartyErrorCloseCta,
      onPrimaryPressed: onClosePressed,
      secondaryButtonCta: context.l10n.generalShowDetailsCta,
      secondaryButtonIcon: const Icon(Icons.info_outline_rounded),
      onSecondaryButtonPressed: !kReleaseMode ? () => ErrorDetailsSheet.show(context, error: error) : null,
      flipButtonOrder: true,
    );
  }
}
