import 'package:clock/clock.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/card/metadata/card_display_metadata.dart';
import '../../../domain/model/card/status/card_status.dart';
import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/model/organization.dart';
import '../mapper.dart';
import 'attribute/card_attribute_mapper.dart';

/// Maps a [Attestation] to a [WalletCard] and enriches with (currently) hardcoded data.
class CardMapper extends Mapper<core.AttestationPresentation, WalletCard> {
  final Mapper<CardAttributeWithCardId, DataAttribute> _attributeMapper;
  final Mapper<core.Organization, Organization> _organizationMapper;
  final Mapper<core.DisplayMetadata, CardDisplayMetadata> _displayMetadataMapper;
  final Mapper<core.RevocationStatus?, CardStatus> _cardStatusMapper;

  CardMapper(
    this._attributeMapper,
    this._organizationMapper,
    this._displayMetadataMapper,
    this._cardStatusMapper,
  );

  @override
  WalletCard map(core.AttestationPresentation input) {
    final String? cardId = switch (input.identity) {
      core.AttestationIdentity_Ephemeral() => null,
      core.AttestationIdentity_Fixed(:final id) => id,
    };

    return WalletCard(
      attestationId: cardId,
      attestationType: input.attestationType,
      issuer: _organizationMapper.map(input.issuer),
      status: _cardStatusMapper.map(input.revocationStatus),
      // TODO(Daan): Implement validFrom & validUntil mapping once Core logic is implemented in [PVW-5161];
      validFrom: clock.now().subtract(const Duration(days: 1)),
      validUntil: clock.now().add(const Duration(days: 365)),
      metadata: _displayMetadataMapper.mapList(input.displayMetadata),
      attributes: _attributeMapper.mapList(
        input.attributes.map(
          (attribute) => CardAttributeWithCardId(
            cardId,
            attribute,
          ),
        ),
      ),
    );
  }
}
