import 'package:flutter/material.dart';

import '../../../../domain/model/wallet_card.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../common/page/legacy_terminal_page.dart';
import '../../../common/widget/stacked_wallet_cards.dart';

class WalletPersonalizeSuccessPage extends StatelessWidget {
  final VoidCallback onContinuePressed;
  final List<WalletCard> cards;

  const WalletPersonalizeSuccessPage({
    required this.onContinuePressed,
    required this.cards,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return LegacyTerminalPage(
      icon: Icons.check,
      title: context.l10n.walletPersonalizeSuccessPageTitle,
      content: Padding(
        padding: const EdgeInsets.only(top: 32, left: 16, right: 16),
        child: Center(
          child: ExcludeSemantics(
            child: StackedWalletCards(
              cards: cards,
              onCardPressed: (card) => onContinuePressed(),
            ),
          ),
        ),
      ),
      description: context.l10n.walletPersonalizeSuccessPageDescription,
      primaryButtonCta: context.l10n.walletPersonalizeSuccessPageContinueCta,
      onPrimaryPressed: onContinuePressed,
    );
  }
}
