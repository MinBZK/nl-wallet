import 'package:flutter/material.dart';

import '../../../domain/model/card_front.dart';
import '../../../domain/model/wallet_card.dart';
import 'card/wallet_card_item.dart';

const _kCardRenderSize = Size(328, 192);
const _kCardDisplaySize = Size(40, 66);

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
                      Text(card.front.title, style: Theme.of(context).textTheme.titleMedium),
                      Text(
                        card.front.subtitle ?? card.front.info ?? '',
                        style: Theme.of(context).textTheme.bodyLarge,
                      ),
                    ],
                  ),
                ),
                Checkbox(
                  value: isSelected,
                  onChanged: (checked) => onCardSelectionToggled(card),
                  fillColor: showError ? MaterialStatePropertyAll(Theme.of(context).colorScheme.error) : null,
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
    return SizedBox.fromSize(
      size: _kCardDisplaySize,
      child: FittedBox(
        alignment: Alignment.center,
        child: SizedBox.fromSize(
          size: _kCardRenderSize,
          child: WalletCardItem.fromCardFront(front: front),
        ),
      ),
    );
  }
}
