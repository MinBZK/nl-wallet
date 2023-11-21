import 'package:wallet_core/core.dart';

import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/card_front.dart';
import '../../../domain/model/wallet_card.dart';
import '../mapper.dart';

/// Maps a [Card] to a [WalletCard] and enriches with (currently) hardcoded data.
class CardMapper extends Mapper<Card, WalletCard> {
  final Mapper<Card, CardFront> _cardFrontMapper;
  final Mapper<CardAttribute, DataAttribute> _attributeMapper;

  CardMapper(this._cardFrontMapper, this._attributeMapper);

  @override
  WalletCard map(Card input) {
    final String cardId = input.persistence.map(
      inMemory: (inMemory) => '',
      stored: (stored) => stored.id,
    );
    return WalletCard(
      id: cardId,
      issuerId: '', // FIXME: Eventually remove issuerId (mock builds still rely on them for now)
      front: _cardFrontMapper.map(input),
      attributes: _attributeMapper.mapList(input.attributes),
    );
  }
}
