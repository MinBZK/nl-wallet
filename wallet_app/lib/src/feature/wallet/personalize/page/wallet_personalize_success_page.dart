import 'package:flutter/material.dart';

import '../../../../domain/model/card/wallet_card.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../common/page/terminal_page.dart';
import '../../../common/widget/stacked_wallet_cards.dart';

class WalletPersonalizeSuccessPage extends StatelessWidget {
  final VoidCallback onContinuePressed;
  final List<WalletCard> cards;

  const WalletPersonalizeSuccessPage({
    required this.onContinuePressed,
    required this.cards,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return TerminalPage(
      title: context.l10n.walletPersonalizeSuccessPageTitle,
      illustration: Padding(
        padding: const EdgeInsets.only(top: 12, left: 16, right: 16),
        child: Center(
          child: StackedWalletCards(
            cards: cards,
            onCardPressed: (card) => onContinuePressed(),
          ),
        ),
      ),
      description: context.l10n.walletPersonalizeSuccessPageDescription,
      primaryButtonCta: context.l10n.walletPersonalizeSuccessPageContinueCta,
      onPrimaryPressed: onContinuePressed,
    );
  }
}
