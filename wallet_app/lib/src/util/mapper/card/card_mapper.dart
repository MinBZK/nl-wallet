import 'package:wallet_core/core.dart' as core;

import '../../../../environment.dart';
import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/card/card_config.dart';
import '../../../domain/model/card/card_front.dart';
import '../../../domain/model/card/metadata/card_display_metadata.dart';
import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/model/organization.dart';
import '../../extension/object_extension.dart';
import '../mapper.dart';
import 'attribute/card_attribute_mapper.dart';

/// Maps a [Attestation] to a [WalletCard] and enriches with (currently) hardcoded data.
class CardMapper extends Mapper<core.Attestation, WalletCard> {
  final Mapper<core.Attestation, CardFront> _cardFrontMapper;
  final Mapper<String /*docType*/, CardConfig> _cardConfigMapper;
  final Mapper<CardAttributeWithDocType, DataAttribute> _attributeMapper;
  final Mapper<core.Organization, Organization> _organizationMapper;
  final Mapper<core.DisplayMetadata, CardDisplayMetadata> _displayMetadataMapper;

  CardMapper(
    this._cardFrontMapper,
    this._cardConfigMapper,
    this._attributeMapper,
    this._organizationMapper,
    this._displayMetadataMapper,
  );

  @override
  WalletCard map(core.Attestation input) {
    final String cardId = switch (input.identity) {
      core.AttestationIdentity_Ephemeral() => input.attestationType,
      core.AttestationIdentity_Fixed(:final id) => id,
    };

    return WalletCard(
      id: cardId,
      docType: input.attestationType,
      issuer: _organizationMapper.map(input.issuer),
      front: _cardFrontMapper.map(input).takeIf((_) => Environment.mockRepositories),
      attributes: _attributeMapper.mapList(
        input.attributes.map(
          (attribute) => CardAttributeWithDocType(
            input.attestationType,
            attribute,
          ),
        ),
      ),
      metadata: _displayMetadataMapper.mapList(input.displayMetadata),
      config: _cardConfigMapper.map(input.attestationType),
    );
  }
}
