import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/util/mapper/card/attribute/card_attribute_mapper.dart';
import 'package:wallet/src/util/mapper/mapper.dart';
import 'package:wallet_core/core.dart';

import '../../../../mocks/wallet_mocks.dart';

const _kSampleCardAttributeLabels = [LocalizedString(language: '-', value: '-')];
const _kSampleCardValue = CardValue_String(value: '-');
const _kSampleCardAttribute = CardAttribute(
  key: 'card.key',
  labels: _kSampleCardAttributeLabels,
  value: _kSampleCardValue,
);

void main() {
  late Mapper<List<LocalizedString>, LocalizedText> mockLabelMapper;
  late Mapper<CardValue, AttributeValue> mockValueMapper;

  late Mapper<CardAttributeWithDocType, DataAttribute> mapper;

  setUp(() {
    mockLabelMapper = MockMapper();
    mockValueMapper = MockMapper();

    mapper = CardAttributeMapper(mockValueMapper, mockLabelMapper);
  });

  group('map', () {
    test('should return `DataAttribute`', () {
      when(mockLabelMapper.map(_kSampleCardAttributeLabels)).thenReturn({'nl': 'Test'});
      when(mockValueMapper.map(_kSampleCardValue)).thenReturn(const StringValue('John Doe'));

      const expected = DataAttribute(
        key: 'card.key',
        label: {'nl': 'Test'},
        value: StringValue('John Doe'),
        sourceCardDocType: 'docType',
      );

      final actual = mapper.map(const CardAttributeWithDocType('docType', _kSampleCardAttribute));
      expect(actual, expected);
    });

    test('should call `map` once on all class dependencies', () {
      mapper.map(const CardAttributeWithDocType('docType', _kSampleCardAttribute));

      verify(mockLabelMapper.map(_kSampleCardAttributeLabels)).called(1);
      verify(mockValueMapper.map(_kSampleCardValue)).called(1);
    });
  });
}
