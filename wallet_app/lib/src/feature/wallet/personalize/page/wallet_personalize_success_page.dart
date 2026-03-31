import 'package:flutter/material.dart';

import '../../../../domain/model/card/wallet_card.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../common/page/terminal_page.dart';
import '../../../common/widget/button/primary_button.dart';
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
      illustration: _buildCardStack(),
      description: context.l10n.walletPersonalizeSuccessPageDescription,
      primaryButton: PrimaryButton(
        text: Text(context.l10n.walletPersonalizeSuccessPageContinueCta),
        onPressed: onContinuePressed,
        key: const Key('primaryButtonCta'),
      ),
    );
  }

  Widget _buildCardStack() {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 0),
      child: Center(
        child: StackedWalletCards(
          cards: cards,
          onCardPressed: (card) => onContinuePressed(),
        ),
      ),
    );
  }
}
