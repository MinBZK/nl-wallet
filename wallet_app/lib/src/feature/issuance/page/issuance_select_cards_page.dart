import 'package:flutter/material.dart';

import '../../../domain/model/wallet_card.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../common/screen/placeholder_screen.dart';
import '../../common/widget/button/confirm/confirm_buttons.dart';
import '../../common/widget/button/link_button.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/secondary_button.dart';
import '../../common/widget/select_card_row.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../common/widget/wallet_scrollbar.dart';

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
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return WalletScrollbar(
      child: CustomScrollView(
        slivers: [
          const SliverSizedBox(height: 24),
          SliverToBoxAdapter(child: _buildHeader(context)),
          const SliverSizedBox(height: 24),
          const SliverToBoxAdapter(child: Divider()),
          SliverList(delegate: _cardBuilderDelegate()),
          SliverToBoxAdapter(child: _buildDataIncorrect(context)),
          SliverFillRemaining(
            hasScrollBody: false,
            fillOverscroll: true,
            child: _buildActionButtons(context),
          ),
        ],
      ),
    );
  }

  Widget _buildHeader(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            context.l10n.issuanceSelectCardsPageTitle,
            style: context.textTheme.displayMedium,
            textAlign: TextAlign.start,
          ),
          const SizedBox(height: 8),
          Text(
            context.l10n.issuanceSelectCardsPageDescription,
            style: context.textTheme.bodyLarge,
            textAlign: TextAlign.start,
          ),
        ],
      ),
    );
  }

  Widget _buildDataIncorrect(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
          child: LinkButton(
            text: Text.rich(context.l10n.issuanceSelectCardsPageDataIncorrectCta.toTextSpan(context)),
            onPressed: () => PlaceholderScreen.showGeneric(context),
          ),
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
    return Align(
      alignment: Alignment.bottomCenter,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (showNoSelectionError) _buildNoSelectionRow(context),
          ConfirmButtons(
            primaryButton: PrimaryButton(
              key: const Key('acceptButton'),
              onPressed: onAddSelectedPressed,
              text: Text.rich(context.l10n.issuanceSelectCardsPageAddCta.toTextSpan(context)),
            ),
            secondaryButton: SecondaryButton(
              key: const Key('rejectButton'),
              onPressed: onStopPressed,
              icon: const Icon(Icons.block),
              text: Text.rich(context.l10n.issuanceSelectCardsPageStopCta.toTextSpan(context)),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildNoSelectionRow(BuildContext context) {
    final errorColor = context.colorScheme.error;
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
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
              context.l10n.issuanceSelectCardsPageNoSelectionError,
              style: context.textTheme.bodyMedium?.copyWith(color: errorColor),
            ),
          ),
        ],
      ),
    );
  }
}
