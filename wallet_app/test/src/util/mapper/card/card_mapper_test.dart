import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/card/card_config.dart';
import 'package:wallet/src/domain/model/card/card_front.dart';
import 'package:wallet/src/domain/model/card/metadata/card_display_metadata.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/domain/model/organization.dart';
import 'package:wallet/src/util/mapper/card/attribute/card_attribute_mapper.dart';
import 'package:wallet/src/util/mapper/card/card_mapper.dart';
import 'package:wallet/src/util/mapper/mapper.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../mocks/core_mock_data.dart';
import '../../../mocks/wallet_mocks.dart';

const _kSampleDocType = 'pid_id';
const _kSampleAttributeName = CoreMockData.attestationAttributeName;
const _kSampleAttributeCity = CoreMockData.attestationAttributeCity;
const _kSampleIssuer = CoreMockData.organization;

const _kSampleCard = core.Attestation(
  identity: core.AttestationIdentity.ephemeral(),
  attestationType: 'com.example.pid',
  displayMetadata: [CoreMockData.enDisplayMetadata],
  attributes: [_kSampleAttributeName, _kSampleAttributeCity],
  issuer: _kSampleIssuer,
);

void main() {
  late Mapper<core.Attestation, CardFront> mockCardFrontMapper;
  late Mapper<CardAttributeWithDocType, DataAttribute> mockCardAttributeMapper;
  late Mapper<String, CardConfig> mockCardConfigMapper;
  late Mapper<core.Organization, Organization> mockOrganizationMapper;
  late Mapper<core.DisplayMetadata, CardDisplayMetadata> mockDisplayMetadataMapper;

  late Mapper<core.Attestation, WalletCard> mapper;

  setUp(() {
    provideDummy<CardConfig>(const CardConfig());

    mockCardFrontMapper = MockMapper();
    mockCardAttributeMapper = MockMapper();
    mockCardConfigMapper = MockMapper();
    mockOrganizationMapper = MockMapper();
    mockDisplayMetadataMapper = MockMapper();

    mapper = CardMapper(
      mockCardFrontMapper,
      mockCardConfigMapper,
      mockCardAttributeMapper,
      mockOrganizationMapper,
      mockDisplayMetadataMapper,
    );
  });

  group('map', () {
    test('card with `InMemory` persistence should return docType as `id`', () {
      final WalletCard actual = mapper.map(_kSampleCard);
      expect(actual.id, _kSampleCard.attestationType);
    });

    test('card with `stored` persistence should return storage `id`', () {
      const input = core.Attestation(
        identity: core.AttestationIdentity.fixed(id: 'id-987'),
        attestationType: _kSampleDocType,
        displayMetadata: [CoreMockData.enDisplayMetadata],
        attributes: [],
        issuer: _kSampleIssuer,
      );
      expect(mapper.map(input).id, 'id-987');
    });

    test('all card attributes should be mapped by the attributeMapper', () {
      mapper.map(_kSampleCard);

      verify(
        mockCardAttributeMapper.mapList(
          _kSampleCard.attributes.map((e) => CardAttributeWithDocType(_kSampleCard.attestationType, e)),
        ),
      ).called(1);
    });

    test('card with `pid_id` docType should call `mockCardFrontMapper` once', () {
      mapper.map(_kSampleCard);

      verify(mockCardFrontMapper.map(_kSampleCard)).called(1);
    });
  });
}
