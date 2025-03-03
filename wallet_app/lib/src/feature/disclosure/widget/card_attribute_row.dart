import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/card/wallet_card.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';

class CardAttributeRow extends StatelessWidget {
  final MapEntry<WalletCard, List<DataAttribute>> entry;

  const CardAttributeRow({
    required this.entry,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        ExcludeSemantics(
          child: Image.asset(WalletAssets.icon_card_share),
        ),
        const SizedBox(width: 16),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                context.l10n.cardAttributeRowTitle(entry.key.title.l10nValue(context)),
                style: context.textTheme.titleMedium,
              ),
              const SizedBox(height: 4),
              ...entry.value.map(
                (attribute) => Text(
                  attribute.label.l10nValue(context),
                  style: context.textTheme.bodyLarge,
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}
