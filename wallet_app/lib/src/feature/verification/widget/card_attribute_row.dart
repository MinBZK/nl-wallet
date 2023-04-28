import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/wallet_card.dart';

const _kCardShareAsset = 'assets/images/ic_card_share.png';

class CardAttributeRow extends StatelessWidget {
  final MapEntry<WalletCard, List<DataAttribute>> entry;

  const CardAttributeRow({required this.entry, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Image.asset(_kCardShareAsset),
        const SizedBox(width: 16),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(locale.cardAttributeRowTitle(entry.key.front.title), style: Theme.of(context).textTheme.titleMedium),
              const SizedBox(height: 4),
              ...entry.value.map(
                (attrib) => Text(
                  attrib.label,
                  style: Theme.of(context).textTheme.bodyLarge,
                ),
              ),
            ],
          ),
        )
      ],
    );
  }
}
