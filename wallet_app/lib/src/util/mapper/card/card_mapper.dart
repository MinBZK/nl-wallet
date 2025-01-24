import 'package:wallet_core/core.dart' as core;

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/card_config.dart';
import '../../../domain/model/card_front.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/wallet_card.dart';
import '../mapper.dart';
import 'attribute/card_attribute_mapper.dart';

/// Maps a [Card] to a [WalletCard] and enriches with (currently) hardcoded data.
class CardMapper extends Mapper<core.Card, WalletCard> {
  final Mapper<core.Card, CardFront> _cardFrontMapper;
  final Mapper<String /*docType*/, CardConfig> _cardConfigMapper;
  final Mapper<CardAttributeWithDocType, DataAttribute> _attributeMapper;
  final Mapper<core.Organization, Organization> _organizationMapper;

  CardMapper(this._cardFrontMapper, this._cardConfigMapper, this._attributeMapper, this._organizationMapper);

  @override
  WalletCard map(core.Card input) {
    final String cardId = input.persistence.map(
      inMemory: (inMemory) => input.docType,
      stored: (stored) => stored.id,
    );
    return WalletCard(
      id: cardId,
      docType: input.docType,
      issuer: _organizationMapper.map(input.issuer),
      front: _cardFrontMapper.map(input),
      attributes: _attributeMapper.mapList(
        input.attributes.map(
          (attribute) => CardAttributeWithDocType(input.docType, attribute),
        ),
      ),
      config: _cardConfigMapper.map(input.docType),
    );
  }
}
