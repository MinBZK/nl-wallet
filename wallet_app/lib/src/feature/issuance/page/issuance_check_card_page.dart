import 'package:flutter/material.dart';

import '../../../domain/model/wallet_card.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/button/confirm_buttons.dart';
import '../../common/page/check_data_offering_page.dart';

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
      attributes: card.attributes,
      title: context.l10n.issuanceCheckCardPageTitle,
      overline: context.l10n.issuanceCheckCardPageOverline(currentCardIndex + 1, totalNrOfCards),
      cardFront: card.front,
      showHeaderAttributesDivider: false,
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return ConfirmButtons(
      onDeclinePressed: onDeclinePressed,
      onAcceptPressed: onAcceptPressed,
      acceptText: context.l10n.issuanceCheckCardPageConfirmCta,
      declineText: context.l10n.issuanceCheckCardPageRejectCta,
    );
  }
}
