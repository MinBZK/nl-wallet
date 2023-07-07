import 'package:flutter/material.dart';

import '../../../../domain/model/card_front.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../common/widget/flow_terminal_page.dart';
import '../../../common/widget/stacked_wallet_cards.dart';

class WalletPersonalizeSuccessPage extends StatelessWidget {
  final VoidCallback onContinuePressed;
  final List<CardFront> cards;

  const WalletPersonalizeSuccessPage({
    required this.onContinuePressed,
    required this.cards,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return FlowTerminalPage(
      icon: Icons.check,
      title: context.l10n.walletPersonalizeSuccessPageTitle,
      content: Padding(
        padding: const EdgeInsets.only(top: 32, left: 16, right: 16),
        child: Center(
          child: ExcludeSemantics(
            child: StackedWalletCards(cards: cards),
          ),
        ),
      ),
      description: context.l10n.walletPersonalizeSuccessPageDescription,
      closeButtonCta: context.l10n.walletPersonalizeSuccessPageContinueCta,
      onClosePressed: onContinuePressed,
    );
  }
}
