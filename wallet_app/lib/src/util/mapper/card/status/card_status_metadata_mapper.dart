import 'package:flutter/material.dart';

import '../../../../domain/model/card/status/card_status_metadata.dart';
import '../../../../domain/model/card/wallet_card.dart';
import '../../../formatter/card/status/card_status_metadata_formatter.dart';
import '../../../formatter/card/status/impl/card_status_metadata_card_data_screen_formatter.dart';
import '../../../formatter/card/status/impl/card_status_metadata_card_detail_screen_formatter.dart';
import '../../../formatter/card/status/impl/card_status_metadata_shared_attributes_card_formatter.dart';
import '../../../formatter/card/status/impl/card_status_metadata_wallet_item_formatter.dart';
import 'card_status_render_type.dart';

class CardStatusMetadataMapper {
  static CardStatusMetadata? map(
    BuildContext context,
    WalletCard card,
    CardStatusRenderType renderType, {
    bool isPidCard = false,
  }) {
    final CardStatusMetadataFormatter formatter;
    switch (renderType) {
      case CardStatusRenderType.walletCardItem:
        formatter = CardStatusMetadataWalletItemFormatter();
      case CardStatusRenderType.cardDetailScreen:
        formatter = CardStatusMetadataCardDetailScreenFormatter();
      case CardStatusRenderType.cardDataScreen:
        formatter = CardStatusMetadataCardDataScreenFormatter();
      case CardStatusRenderType.sharedAttributesCard:
        formatter = CardStatusMetadataSharedAttributesCardFormatter();
    }

    if (formatter.show(card.status)) {
      return CardStatusMetadata(
        text: formatter.text(context, card, isPidCard: isPidCard),
        textColor: formatter.textColor(context, card.status),
        icon: formatter.icon(card.status),
        iconColor: formatter.iconColor(context, card.status),
        backgroundColor: formatter.backgroundColor(context, card.status),
      );
    } else {
      return null;
    }
  }
}
