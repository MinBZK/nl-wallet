import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/card_config.dart';
import 'package:wallet/src/domain/model/card_front.dart';
import 'package:wallet/src/domain/model/organization.dart';
import 'package:wallet/src/domain/model/wallet_card.dart';
import 'package:wallet/src/util/mapper/card/attribute/card_attribute_mapper.dart';
import 'package:wallet/src/util/mapper/card/card_mapper.dart';
import 'package:wallet/src/util/mapper/mapper.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../mocks/core_mock_data.dart';
import '../../../mocks/wallet_mocks.dart';

const _kSampleDocType = 'pid_id';
const _kSampleCardAttributeName = CoreMockData.cardAttributeName;
const _kSampleCardAttributeCity = CoreMockData.cardAttributeCity;
const _kSampleIssuer = CoreMockData.organization;

const _kSampleCard = core.Card(
  persistence: core.CardPersistence_InMemory(),
  docType: _kSampleDocType,
  attributes: [_kSampleCardAttributeName, _kSampleCardAttributeCity],
  issuer: _kSampleIssuer,
);

void main() {
  late Mapper<core.Card, CardFront> mockCardFrontMapper;
  late Mapper<CardAttributeWithDocType, DataAttribute> mockCardAttributeMapper;
  late Mapper<String, CardConfig> mockCardConfigMapper;
  late Mapper<core.Organization, Organization> mockOrganizationMapper;

  late Mapper<core.Card, WalletCard> mapper;

  setUp(() {
    provideDummy<CardConfig>(const CardConfig());

    mockCardFrontMapper = MockMapper();
    mockCardAttributeMapper = MockMapper();
    mockCardConfigMapper = MockMapper();
    mockOrganizationMapper = MockMapper();

    mapper = CardMapper(
      mockCardFrontMapper,
      mockCardConfigMapper,
      mockCardAttributeMapper,
      mockOrganizationMapper,
    );
  });

  group('map', () {
    test('card with `InMemory` persistence should return docType as `id`', () {
      final WalletCard actual = mapper.map(_kSampleCard);
      expect(actual.id, _kSampleCard.docType);
    });

    test('card with `stored` persistence should return storage `id`', () {
      const input = core.Card(
        persistence: core.CardPersistence_Stored(id: 'id-987'),
        docType: _kSampleDocType,
        attributes: [],
        issuer: _kSampleIssuer,
      );
      expect(mapper.map(input).id, 'id-987');
    });

    test('all card attributes should be mapped by the attributeMapper', () {
      mapper.map(_kSampleCard);

      verify(
        mockCardAttributeMapper.mapList(
          _kSampleCard.attributes.map((e) => CardAttributeWithDocType(_kSampleCard.docType, e)),
        ),
      ).called(1);
    });

    test('card with `pid_id` docType should call `mockCardFrontMapper` once', () {
      mapper.map(_kSampleCard);

      verify(mockCardFrontMapper.map(_kSampleCard)).called(1);
    });
  });
}
