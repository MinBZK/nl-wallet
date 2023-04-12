import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/wallet_card.dart';
import '../../common/widget/check_data_offering_page.dart';
import '../../common/widget/confirm_buttons.dart';

class IssuanceCheckCardPage extends StatelessWidget {
  final VoidCallback onDecline;
  final VoidCallback onAccept;
  final WalletCard card;

  // Provide information needed to generate the overline, i.e. 'Card x of y'
  final int totalNrOfCards, currentCardIndex;

  const IssuanceCheckCardPage({
    required this.onDecline,
    required this.onAccept,
    required this.card,
    required this.totalNrOfCards,
    required this.currentCardIndex,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return CheckDataOfferingPage(
      bottomSection: _buildBottomSection(context),
      attributes: card.attributes,
      title: locale.issuanceCheckCardPageTitle,
      overline: locale.issuanceCheckCardPageOverline(currentCardIndex + 1, totalNrOfCards),
      cardFront: card.front,
      showHeaderAttributesDivider: false,
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return ConfirmButtons(
      onDecline: onDecline,
      onAccept: onAccept,
      acceptText: locale.issuanceCheckCardPageConfirmCta,
      declineText: locale.issuanceCheckCardPageRejectCta,
    );
  }
}
