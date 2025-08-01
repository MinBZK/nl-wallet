import 'package:flutter/cupertino.dart';

import '../../../domain/model/card/wallet_card.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/page/terminal_page.dart';
import '../../common/widget/stacked_wallet_cards.dart';

class RenewPidSuccessPage extends StatelessWidget {
  final List<WalletCard> cards;
  final VoidCallback onPrimaryPressed;

  const RenewPidSuccessPage({
    required this.cards,
    required this.onPrimaryPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return TerminalPage(
      title: context.l10n.renewPidSuccessPageTitle,
      description: context.l10n.renewPidSuccessPageDescription,
      primaryButtonCta: context.l10n.renewPidSuccessPageToDashboardCta,
      onPrimaryPressed: onPrimaryPressed,
      illustration: Padding(
        padding: const EdgeInsets.only(top: 32, left: 16, right: 16),
        child: Center(
          child: StackedWalletCards(
            cards: cards,
            onCardPressed: (card) => onPrimaryPressed(),
          ),
        ),
      ),
    );
  }
}
