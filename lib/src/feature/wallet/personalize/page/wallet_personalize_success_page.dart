import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../../domain/model/card_front.dart';
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
    final locale = AppLocalizations.of(context);
    return FlowTerminalPage(
      icon: Icons.check,
      title: locale.walletPersonalizeSuccessPageTitle,
      content: Column(
        children: [
          const SizedBox(height: 16),
          Padding(
            padding: const EdgeInsets.all(16),
            child: StackedWalletCards(cards: cards),
          ),
        ],
      ),
      description: locale.walletPersonalizeSuccessPageDescription,
      closeButtonCta: locale.walletPersonalizeSuccessPageContinueCta,
      onClosePressed: onContinuePressed,
    );
  }
}
