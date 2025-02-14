import 'package:wallet_core/core.dart' as core;

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/card_config.dart';
import '../../../domain/model/card_front.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/wallet_card.dart';
import '../mapper.dart';
import 'attribute/card_attribute_mapper.dart';

/// Maps a [Attestation] to a [WalletCard] and enriches with (currently) hardcoded data.
class CardMapper extends Mapper<core.Attestation, WalletCard> {
  final Mapper<core.Attestation, CardFront> _cardFrontMapper;
  final Mapper<String /*docType*/, CardConfig> _cardConfigMapper;
  final Mapper<CardAttributeWithDocType, DataAttribute> _attributeMapper;
  final Mapper<core.Organization, Organization> _organizationMapper;

  CardMapper(this._cardFrontMapper, this._cardConfigMapper, this._attributeMapper, this._organizationMapper);

  @override
  WalletCard map(core.Attestation input) {
    final String cardId = input.identity.map(
      ephemeral: (ephemeral) => input.attestationType,
      fixed: (fixed) => fixed.id,
    );
    return WalletCard(
      id: cardId,
      docType: input.attestationType,
      issuer: _organizationMapper.map(input.issuer),
      front: _cardFrontMapper.map(input),
      attributes: _attributeMapper.mapList(
        input.attributes.map(
          (attribute) => CardAttributeWithDocType(input.attestationType, attribute),
        ),
      ),
      config: _cardConfigMapper.map(input.attestationType),
    );
  }
}
