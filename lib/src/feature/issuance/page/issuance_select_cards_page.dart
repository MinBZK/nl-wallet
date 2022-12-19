import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/wallet_card.dart';
import '../../common/widget/confirm_buttons.dart';
import '../../common/widget/link_button.dart';
import '../../common/widget/placeholder_screen.dart';
import '../../common/widget/select_card_row.dart';
import '../../common/widget/sliver_sized_box.dart';

class IssuanceSelectCardsPage extends StatelessWidget {
  final List<WalletCard> cards;
  final List<String> selectedCardIds;
  final Function(WalletCard) onCardSelectionToggled;
  final VoidCallback onStopPressed;
  final VoidCallback onAddSelectedPressed;
  final bool showNoSelectionError;

  const IssuanceSelectCardsPage({
    required this.cards,
    required this.selectedCardIds,
    required this.onCardSelectionToggled,
    required this.onStopPressed,
    required this.onAddSelectedPressed,
    this.showNoSelectionError = false,
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
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            locale.issuanceSelectCardsPageTitle,
            style: Theme.of(context).textTheme.headline2,
            textAlign: TextAlign.start,
          ),
          const SizedBox(height: 8),
          Text(
            locale.issuanceSelectCardsPageDescription,
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
          child: Text(locale.issuanceSelectCardsPageDataIncorrectCta),
          onPressed: () => PlaceholderScreen.show(context, locale.issuanceSelectCardsPageDataIncorrectCta),
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
          showError: showNoSelectionError,
        );
      },
      childCount: cards.length,
    );
  }

  Widget _buildActionButtons(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Align(
      alignment: Alignment.bottomCenter,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (showNoSelectionError) _buildNoSelectionRow(context),
          ConfirmButtons(
            onDecline: onStopPressed,
            onAccept: onAddSelectedPressed,
            acceptText: locale.issuanceSelectCardsPageAddCta,
            declineText: locale.issuanceSelectCardsPageStopCta,
            acceptIcon: Icons.arrow_forward,
            declineIcon: Icons.block,
          )
        ],
      ),
    );
  }

  Widget _buildNoSelectionRow(BuildContext context) {
    final errorColor = Theme.of(context).errorColor;
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16.0),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          Icon(
            Icons.error_outline,
            color: errorColor,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              AppLocalizations.of(context).issuanceSelectCardsPageNoSelectionError,
              style: Theme.of(context).textTheme.bodyText2?.copyWith(color: errorColor),
            ),
          )
        ],
      ),
    );
  }
}
