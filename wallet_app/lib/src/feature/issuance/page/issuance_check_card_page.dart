import 'package:flutter/material.dart';

import '../../../domain/model/card/wallet_card.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../common/page/check_data_offering_page.dart';
import '../../common/widget/button/confirm/confirm_buttons.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/secondary_button.dart';

class IssuanceCheckCardPage extends StatelessWidget {
  final VoidCallback onDeclinePressed;
  final VoidCallback onAcceptPressed;
  final WalletCard card;

  // Provide information needed to generate the overline, i.e. 'Card x of y'
  final int totalNrOfCards, currentCardIndex;

  const IssuanceCheckCardPage({
    required this.onDeclinePressed,
    required this.onAcceptPressed,
    required this.card,
    required this.totalNrOfCards,
    required this.currentCardIndex,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return CheckDataOfferingPage(
      bottomSection: _buildBottomSection(context),
      offeredCard: card,
      title: context.l10n.issuanceCheckCardPageTitle,
      overline: context.l10n.issuanceCheckCardPageOverline(currentCardIndex + 1, totalNrOfCards),
      showHeaderAttributesDivider: false,
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return ConfirmButtons(
      primaryButton: PrimaryButton(
        key: const Key('acceptButton'),
        onPressed: onAcceptPressed,
        text: Text.rich(context.l10n.issuanceCheckCardPageConfirmCta.toTextSpan(context)),
        icon: null,
      ),
      secondaryButton: SecondaryButton(
        key: const Key('rejectButton'),
        onPressed: onDeclinePressed,
        icon: const Icon(Icons.block_flipped),
        text: Text.rich(context.l10n.issuanceCheckCardPageRejectCta.toTextSpan(context)),
      ),
    );
  }
}
