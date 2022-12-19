import 'package:flutter/material.dart';

import '../../../domain/model/card_front.dart';
import '../../../domain/model/wallet_card.dart';
import 'wallet_card_front.dart';

class SelectCardRow extends StatelessWidget {
  final Function(WalletCard) onCardSelectionToggled;
  final WalletCard card;
  final bool isSelected;
  final bool showError;

  const SelectCardRow({
    required this.onCardSelectionToggled,
    required this.card,
    required this.isSelected,
    this.showError = false,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Container(
          padding: const EdgeInsets.symmetric(vertical: 12),
          constraints: const BoxConstraints(minHeight: 96),
          child: InkWell(
            child: Row(
              children: [
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: _buildSizedCardFront(card.front),
                ),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(card.front.title, style: Theme.of(context).textTheme.subtitle1),
                      Text(
                        card.front.subtitle ?? card.front.info ?? '',
                        style: Theme.of(context).textTheme.bodyText1,
                      ),
                    ],
                  ),
                ),
                Checkbox(
                  value: isSelected,
                  onChanged: (checked) => onCardSelectionToggled(card),
                  fillColor: showError ? MaterialStatePropertyAll(Theme.of(context).errorColor) : null,
                ),
                const SizedBox(width: 8),
              ],
            ),
          ),
        ),
        const Divider(height: 1),
      ],
    );
  }

  Widget _buildSizedCardFront(CardFront front) {
    return SizedBox(
      width: 40,
      height: 66,
      child: FittedBox(
        alignment: Alignment.center,
        child: SizedBox(
          height: kWalletCardHeight,
          child: WalletCardFront(cardFront: front, onPressed: null),
        ),
      ),
    );
  }
}
