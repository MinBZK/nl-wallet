import 'package:flutter/material.dart';

import '../../../domain/model/organization.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../common/page/terminal_page.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/secondary_button.dart';

const kConfirmIssuerAcceptButtonKey = Key('confirmIssuerAcceptButton');
const kConfirmIssuerDeclineButtonKey = Key('confirmIssuerDeclineButton');

/// Asks the user to confirm receiving cards from an issuer that does not provide previews.
class IssuanceConfirmIssuerPage extends StatelessWidget {
  final Organization organization;
  final VoidCallback onAccept;
  final VoidCallback onDecline;

  const IssuanceConfirmIssuerPage({
    required this.organization,
    required this.onAccept,
    required this.onDecline,
    super.key,
  });

  static String resolveTitle(BuildContext context, Organization organization) =>
      context.l10n.issuanceConfirmIssuerPageTitle(organization.displayName);

  @override
  Widget build(BuildContext context) {
    return TerminalPage(
      title: resolveTitle(context, organization),
      // TODO(PVW-5559): update the copy to the Figma design once the issued cards are reviewed before being added.
      description: context.l10n.issuanceConfirmIssuerPageDescription,
      primaryButton: PrimaryButton(
        key: kConfirmIssuerAcceptButtonKey,
        onPressed: onAccept,
        text: Text.rich(context.l10n.issuanceConfirmIssuerPageAcceptCta.toTextSpan(context)),
      ),
      secondaryButton: SecondaryButton(
        key: kConfirmIssuerDeclineButtonKey,
        onPressed: onDecline,
        text: Text.rich(context.l10n.issuanceConfirmIssuerPageDeclineCta.toTextSpan(context)),
        icon: const Icon(Icons.block_flipped),
      ),
    );
  }
}
