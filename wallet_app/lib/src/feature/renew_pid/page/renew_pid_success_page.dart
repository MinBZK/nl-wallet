import 'package:flutter/cupertino.dart';

import '../../../domain/model/card/wallet_card.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/page/terminal_page.dart';
import '../../common/widget/button/primary_button.dart';
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
      illustration: _buildCardStack(),
      primaryButton: PrimaryButton(
        text: Text(context.l10n.renewPidSuccessPageToDashboardCta),
        onPressed: onPrimaryPressed,
        key: const Key('primaryButtonCta'),
      ),
    );
  }

  Widget _buildCardStack() {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 32, 16, 0),
      child: Center(
        child: StackedWalletCards(
          cards: cards,
          onCardPressed: (card) => onPrimaryPressed(),
        ),
      ),
    );
  }
}
