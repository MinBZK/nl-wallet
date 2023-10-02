import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/bridge_generated.dart';
import 'package:wallet/src/domain/model/attribute/data_attribute.dart';
import 'package:wallet/src/util/mapper/card/attribute/card_attribute_mapper.dart';
import 'package:wallet/src/util/mapper/locale_mapper.dart';

import '../../../../mocks/wallet_mocks.dart';

const _kSampleCardAttributeLabels = [LocalizedString(language: '-', value: '-')];
const _kSampleCardValue = CardValue_String(value: '-');
const _kSampleCardAttribute = CardAttribute(
  key: 'card.key',
  labels: _kSampleCardAttributeLabels,
  value: _kSampleCardValue,
);

const _kSampleLocale = Locale('-');

void main() {
  late MockLocaleMapper<List<LocalizedString>, String> mockLabelMapper;
  late MockLocaleMapper<CardValue, String> mockValueMapper;

  late LocaleMapper<CardAttribute, DataAttribute> mapper;

  setUp(() {
    mockLabelMapper = MockLocaleMapper();
    mockValueMapper = MockLocaleMapper();

    mapper = CardAttributeMapper(mockLabelMapper, mockValueMapper);
  });

  group('map', () {
    test('should return `DataAttribute`', () {
      when(mockLabelMapper.map(_kSampleLocale, _kSampleCardAttributeLabels)).thenReturn('Language');
      when(mockValueMapper.map(_kSampleLocale, _kSampleCardValue)).thenReturn('Dutch');

      const expected = DataAttribute(
        key: 'card.key',
        label: 'Language',
        value: 'Dutch',
        sourceCardId: '',
        valueType: AttributeValueType.text,
      );
      expect(mapper.map(_kSampleLocale, _kSampleCardAttribute), expected);
    });

    test('should call `map` once on all class dependencies', () {
      mapper.map(_kSampleLocale, _kSampleCardAttribute);

      verify(mockLabelMapper.map(_kSampleLocale, _kSampleCardAttributeLabels)).called(1);
      verify(mockValueMapper.map(_kSampleLocale, _kSampleCardValue)).called(1);
    });
  });
}
