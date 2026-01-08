import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/card/metadata/card_display_metadata.dart';
import 'package:wallet/src/domain/model/card/status/card_status.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/domain/model/organization.dart';
import 'package:wallet/src/util/mapper/card/attribute/card_attribute_mapper.dart';
import 'package:wallet/src/util/mapper/card/card_mapper.dart';
import 'package:wallet/src/util/mapper/mapper.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../mocks/core_mock_data.dart';
import '../../../mocks/wallet_mocks.dart';

const _kSampleAttestationType = 'pid_id';
const _kSampleAttributeName = CoreMockData.attestationAttributeName;
const _kSampleAttributeCity = CoreMockData.attestationAttributeCity;
const _kSampleIssuer = CoreMockData.organization;

const _kSampleCard = core.AttestationPresentation(
  identity: core.AttestationIdentity.ephemeral(),
  attestationType: 'urn:eudi:pid:nl:1',
  displayMetadata: [CoreMockData.enDisplayMetadata],
  attributes: [_kSampleAttributeName, _kSampleAttributeCity],
  issuer: _kSampleIssuer,
  validityStatus: core.ValidityStatus.Valid,
  validityWindow: core.ValidityWindow(validFrom: null, validUntil: null),
);

void main() {
  late Mapper<CardAttributeWithCardId, DataAttribute> mockCardAttributeMapper;
  late Mapper<core.Organization, Organization> mockOrganizationMapper;
  late Mapper<core.DisplayMetadata, CardDisplayMetadata> mockDisplayMetadataMapper;
  late Mapper<core.AttestationPresentation, CardStatus> mockCardStatusMapper;

  late Mapper<core.AttestationPresentation, WalletCard> mapper;

  setUp(() {
    mockCardAttributeMapper = MockMapper();
    mockOrganizationMapper = MockMapper();
    mockDisplayMetadataMapper = MockMapper();
    mockCardStatusMapper = MockMapper();

    mapper = CardMapper(
      mockCardAttributeMapper,
      mockOrganizationMapper,
      mockDisplayMetadataMapper,
      mockCardStatusMapper,
    );
  });

  group('map', () {
    test('card with `InMemory` persistence should return null as `id`', () {
      final WalletCard actual = mapper.map(_kSampleCard);
      expect(actual.attestationId, isNull);
    });

    test('card with `stored` persistence should return storage `id`', () {
      const input = core.AttestationPresentation(
        identity: core.AttestationIdentity.fixed(id: 'id-987'),
        attestationType: _kSampleAttestationType,
        displayMetadata: [CoreMockData.enDisplayMetadata],
        attributes: [],
        issuer: _kSampleIssuer,
        validityStatus: core.ValidityStatus.Valid,
        validityWindow: core.ValidityWindow(validFrom: null, validUntil: null),
      );
      expect(mapper.map(input).attestationId, 'id-987');
    });

    test('all card attributes should be mapped by the attributeMapper', () {
      mapper.map(_kSampleCard);

      verify(
        mockCardAttributeMapper.mapList(
          _kSampleCard.attributes.map((e) => CardAttributeWithCardId(null, e)),
        ),
      ).called(1);
    });
  });
}
