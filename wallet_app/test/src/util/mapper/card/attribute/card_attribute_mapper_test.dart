import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/util/mapper/card/attribute/card_attribute_mapper.dart';
import 'package:wallet/src/util/mapper/mapper.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../../mocks/wallet_mocks.dart';

const _kSampleCardAttributeLabels = [core.ClaimDisplayMetadata(lang: '-', label: '-')];
const _kSampleCardValue = core.AttributeValue_String(value: '-');
const _kSampleCardAttribute = core.AttestationAttribute(
  key: 'card.key',
  svgId: 'card.svgId',
  labels: _kSampleCardAttributeLabels,
  value: _kSampleCardValue,
);

void main() {
  late Mapper<List<core.ClaimDisplayMetadata>, LocalizedText> mockLabelMapper;
  late Mapper<core.AttributeValue, AttributeValue> mockValueMapper;

  late Mapper<CardAttributeWithCardId, DataAttribute> mapper;

  setUp(() {
    mockLabelMapper = MockMapper();
    mockValueMapper = MockMapper();

    mapper = CardAttributeMapper(mockValueMapper, mockLabelMapper);
  });

  group('map', () {
    test('should return `DataAttribute`', () {
      when(mockLabelMapper.map(_kSampleCardAttributeLabels)).thenReturn({const Locale('nl'): 'Test'});
      when(mockValueMapper.map(_kSampleCardValue)).thenReturn(const StringValue('John Doe'));

      final expected = DataAttribute(
        key: 'card.key',
        svgId: 'card.svgId',
        label: {const Locale('nl'): 'Test'},
        value: const StringValue('John Doe'),
      );

      final actual = mapper.map(const CardAttributeWithCardId('docType', _kSampleCardAttribute));
      expect(actual, expected);
    });

    test('should call `map` once on all class dependencies', () {
      mapper.map(const CardAttributeWithCardId('docType', _kSampleCardAttribute));

      verify(mockLabelMapper.map(_kSampleCardAttributeLabels)).called(1);
      verify(mockValueMapper.map(_kSampleCardValue)).called(1);
    });
  });
}
