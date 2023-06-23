import 'package:flutter/material.dart';

import '../../../domain/model/wallet_card.dart';
import '../../../util/extension/build_context_extension.dart';
import 'card/wallet_card_item.dart';

const _kCardDisplayWidth = 40.0;

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
                  child: SizedBox(
                    width: _kCardDisplayWidth,
                    child: WalletCardItem.fromCardFront(
                      front: card.front,
                    ),
                  ),
                ),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(card.front.title, style: context.textTheme.titleMedium),
                      Text(
                        card.front.subtitle ?? card.front.info ?? '',
                        style: context.textTheme.bodyLarge,
                      ),
                    ],
                  ),
                ),
                Checkbox(
                  value: isSelected,
                  onChanged: (checked) => onCardSelectionToggled(card),
                  fillColor: showError ? MaterialStatePropertyAll(context.colorScheme.error) : null,
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
}
