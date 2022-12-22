import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../../domain/model/wallet_card.dart';
import '../../../common/widget/link_button.dart';
import '../../../common/widget/placeholder_screen.dart';
import '../../../common/widget/select_card_row.dart';
import '../../../common/widget/sliver_sized_box.dart';
import '../../../common/widget/text_icon_button.dart';

class WalletPersonalizeSelectCardsPage extends StatelessWidget {
  final List<WalletCard> cards;
  final List<String> selectedCardIds;
  final Function(WalletCard) onCardSelectionToggled;
  final VoidCallback onSkipPressed;
  final VoidCallback onAddSelectedPressed;

  const WalletPersonalizeSelectCardsPage({
    required this.cards,
    required this.selectedCardIds,
    required this.onCardSelectionToggled,
    required this.onSkipPressed,
    required this.onAddSelectedPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scrollbar(
      child: CustomScrollView(
        slivers: [
          const SliverSizedBox(height: 24),
          SliverToBoxAdapter(child: _buildHeader(context)),
          const SliverSizedBox(height: 24),
          const SliverToBoxAdapter(child: Divider(height: 1)),
          SliverList(delegate: _cardBuilderDelegate()),
          SliverToBoxAdapter(child: _buildDataIncorrect(context)),
          SliverFillRemaining(
            hasScrollBody: false,
            fillOverscroll: true,
            child: _buildActionButtons(context),
          )
        ],
      ),
    );
  }

  Widget _buildHeader(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            locale.walletPersonalizeSelectCardsPageTitle,
            style: Theme.of(context).textTheme.headline2,
            textAlign: TextAlign.start,
          ),
          const SizedBox(height: 8),
          Text(
            locale.walletPersonalizeSelectCardsPageDescription,
            style: Theme.of(context).textTheme.bodyText1,
            textAlign: TextAlign.start,
          ),
        ],
      ),
    );
  }

  Widget _buildDataIncorrect(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        LinkButton(
          customPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
          child: Text(locale.walletPersonalizeSelectCardsPageDataIncorrectCta),
          onPressed: () => PlaceholderScreen.show(context, locale.walletPersonalizeSelectCardsPageDataIncorrectCta),
        ),
        const Divider(
          height: 1,
        ),
      ],
    );
  }

  SliverChildDelegate _cardBuilderDelegate() {
    return SliverChildBuilderDelegate(
      (context, index) {
        final card = cards[index];
        final isSelected = selectedCardIds.contains(card.id);
        return SelectCardRow(
          onCardSelectionToggled: onCardSelectionToggled,
          card: card,
          isSelected: isSelected,
        );
      },
      childCount: cards.length,
    );
  }

  Widget _buildActionButtons(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      alignment: Alignment.bottomCenter,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          ElevatedButton(
            onPressed: onAddSelectedPressed,
            child: Text(locale.walletPersonalizeSelectCardsPageAddCta),
          ),
          const SizedBox(height: 16),
          Center(
            child: TextIconButton(
              onPressed: onSkipPressed,
              child: Text(locale.walletPersonalizeSelectCardsPageSkipCta),
            ),
          ),
        ],
      ),
    );
  }
}