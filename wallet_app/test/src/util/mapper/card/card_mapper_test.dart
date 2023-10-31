import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/bridge_generated.dart';
import 'package:wallet/src/domain/model/attribute/data_attribute.dart';
import 'package:wallet/src/domain/model/card_front.dart';
import 'package:wallet/src/domain/model/wallet_card.dart';
import 'package:wallet/src/util/mapper/card/card_mapper.dart';
import 'package:wallet/src/util/mapper/mapper.dart';

import '../../../mocks/wallet_mocks.dart';

const _kSampleDocType = 'pid_id';
const _kSampleCardAttributeName = CardAttribute(key: 'name', labels: [], value: CardValue_String(value: 'Willeke'));
const _kSampleCardAttributeCity = CardAttribute(key: 'city', labels: [], value: CardValue_String(value: 'Den Haag'));

const _kSampleCard = Card(
  persistence: CardPersistence_InMemory(),
  docType: _kSampleDocType,
  attributes: [_kSampleCardAttributeName, _kSampleCardAttributeCity],
);

void main() {
  late Mapper<Card, CardFront> mockCardFrontMapper;
  late Mapper<CardAttribute, DataAttribute> mockCardAttributeMapper;

  late Mapper<Card, WalletCard> mapper;

  setUp(() {
    mockCardFrontMapper = MockMapper();
    mockCardAttributeMapper = MockMapper();

    mapper = CardMapper(mockCardFrontMapper, mockCardAttributeMapper);
  });

  group('map', () {
    test('card with `InMemory` persistence should return empty `id`', () {
      WalletCard actual = mapper.map(_kSampleCard);
      expect(actual.id, '');
    });

    test('card with `stored` persistence should return storage `id`', () {
      const input = Card(persistence: CardPersistence_Stored(id: 'id-987'), docType: _kSampleDocType, attributes: []);
      expect(mapper.map(input).id, 'id-987');
    });

    test('card with multiple `attributes` should call mocked `mockCardAttributeMapper` multiple times', () {
      mapper.map(_kSampleCard);

      verify(mockCardAttributeMapper.map(_kSampleCardAttributeName)).called(1);
      verify(mockCardAttributeMapper.map(_kSampleCardAttributeCity)).called(1);
    });

    test('card with `pid_id` docType should call `mockCardFrontMapper` once', () {
      mapper.map(_kSampleCard);

      verify(mockCardFrontMapper.map(_kSampleCard)).called(1);
    });
  });
}
