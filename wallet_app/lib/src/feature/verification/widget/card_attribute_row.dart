import 'package:flutter/material.dart';

import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../util/extension/build_context_extension.dart';

const _kCardShareAsset = 'assets/images/ic_card_share.png';

class CardAttributeRow extends StatelessWidget {
  final MapEntry<WalletCard, List<DataAttribute>> entry;

  const CardAttributeRow({
    required this.entry,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        ExcludeSemantics(
          child: Image.asset(_kCardShareAsset),
        ),
        const SizedBox(width: 16),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                context.l10n.cardAttributeRowTitle(entry.key.front.title),
                style: context.textTheme.titleMedium,
              ),
              const SizedBox(height: 4),
              ...entry.value.map(
                (attribute) => Text(
                  attribute.label,
                  style: context.textTheme.bodyLarge,
                ),
              ),
            ],
          ),
        )
      ],
    );
  }
}
