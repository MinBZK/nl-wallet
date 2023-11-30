import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/util/mapper/card/card_subtitle_mapper.dart';
import 'package:wallet/src/util/mapper/mapper.dart';
import 'package:wallet_core/core.dart';

import '../../../mocks/wallet_mocks.dart';

const _kSampleCardAttributeName = CardAttribute(key: 'name', labels: [], value: CardValue_String(value: 'Willeke'));
const _kSampleCardAttributeCity = CardAttribute(key: 'city', labels: [], value: CardValue_String(value: 'Den Haag'));
const _kSampleNameSubtitle = {'en': 'Willeke', 'nl': 'Willeke'};
const _kSampleCitySubtitle = {'en': 'Den Haag', 'nl': 'Den Haag'};

void main() {
  late Mapper<CardValue, AttributeValue> mockAttributeValueMapper;

  late Mapper<Card, LocalizedText?> mapper;

  setUp(() {
    mockAttributeValueMapper = MockMapper();
    mapper = CardSubtitleMapper(mockAttributeValueMapper);
  });

  Card createSampleCard(String docType, List<CardAttribute> attributes) {
    return Card(persistence: const CardPersistence_InMemory(), docType: docType, attributes: attributes);
  }

  group('map', () {
    test('card with `com.example.pid` docType should return `name` attribute string', () {
      when(mockAttributeValueMapper.map(_kSampleCardAttributeName.value)).thenReturn(const StringValue('Willeke'));

      Card input = createSampleCard('com.example.pid', [_kSampleCardAttributeName, _kSampleCardAttributeCity]);
      expect(mapper.map(input), _kSampleNameSubtitle);

      // Check if every supported locale is mapped to a value
      verify(mockAttributeValueMapper.map(_kSampleCardAttributeName.value))
          .called(AppLocalizations.supportedLocales.length);
    });

    test('card with `com.example.pid` docType should return `name` attribute string', () {
      when(mockAttributeValueMapper.map(_kSampleCardAttributeName.value)).thenReturn(const StringValue('Willeke'));

      Card input = createSampleCard('com.example.pid', [_kSampleCardAttributeName, _kSampleCardAttributeCity]);
      expect(mapper.map(input), _kSampleNameSubtitle);

      // Check if every supported locale is mapped to a value
      verify(mockAttributeValueMapper.map(_kSampleCardAttributeName.value))
          .called(AppLocalizations.supportedLocales.length);
    });

    test('`com.example.pid` card without `name` attribute should not return any subtitle', () {
      Card input = createSampleCard('com.example.pid', [_kSampleCardAttributeCity]);
      expect(mapper.map(input), null);

      verifyNever(mockAttributeValueMapper.map(_kSampleCardAttributeName.value));
    });

    test('card with `com.example.address` docType should return `city` attribute string', () {
      when(mockAttributeValueMapper.map(_kSampleCardAttributeCity.value)).thenReturn(const StringValue('Den Haag'));

      Card input = createSampleCard('com.example.address', [_kSampleCardAttributeName, _kSampleCardAttributeCity]);
      expect(mapper.map(input), _kSampleCitySubtitle);

      // Check if every supported locale is mapped to a value
      verify(mockAttributeValueMapper.map(_kSampleCardAttributeCity.value))
          .called(AppLocalizations.supportedLocales.length);
    });

    test('`com.example.address` card without `city` attribute should not return any subtitle', () {
      Card input = createSampleCard('com.example.address', [_kSampleCardAttributeName]);
      expect(mapper.map(input), null);

      verifyNever(mockAttributeValueMapper.map(_kSampleCardAttributeName.value));
    });

    test('card with unknown docType should not return any subtitle', () {
      Card input = createSampleCard('invalid_doctype', [_kSampleCardAttributeName, _kSampleCardAttributeCity]);
      expect(mapper.map(input), null);
    });
  });
}
