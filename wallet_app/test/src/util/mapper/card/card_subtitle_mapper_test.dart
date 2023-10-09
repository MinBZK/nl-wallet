import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/bridge_generated.dart';
import 'package:wallet/src/util/mapper/card/card_subtitle_mapper.dart';
import 'package:wallet/src/util/mapper/locale_mapper.dart';

import '../../../mocks/wallet_mocks.dart';

const _kSampleLocale = Locale('nl');

const _kSampleCardAttributeName = CardAttribute(key: 'name', labels: [], value: CardValue_String(value: 'Willeke'));
const _kSampleCardAttributeCity = CardAttribute(key: 'city', labels: [], value: CardValue_String(value: 'Den Haag'));

void main() {
  late LocaleMapper<CardValue, String> mockAttributeValueMapper;

  late LocaleMapper<Card, String> mapper;

  setUp(() {
    mockAttributeValueMapper = MockLocaleMapper();

    mapper = CardSubtitleMapper(mockAttributeValueMapper);
  });

  Card createSampleCard(String docType, List<CardAttribute> attributes) {
    return Card(persistence: const CardPersistence_InMemory(), docType: docType, attributes: attributes);
  }

  group('map', () {
    test('card with `com.example.pid` docType should return `name` attribute string', () {
      when(mockAttributeValueMapper.map(_kSampleLocale, _kSampleCardAttributeName.value)).thenReturn('Willeke');

      Card input = createSampleCard('com.example.pid', [_kSampleCardAttributeName, _kSampleCardAttributeCity]);
      expect(mapper.map(_kSampleLocale, input), 'Willeke');

      verify(mockAttributeValueMapper.map(_kSampleLocale, _kSampleCardAttributeName.value)).called(1);
    });

    test('card with `pid_id` docType should return `name` attribute string', () {
      when(mockAttributeValueMapper.map(_kSampleLocale, _kSampleCardAttributeName.value)).thenReturn('Willeke');

      Card input = createSampleCard('pid_id', [_kSampleCardAttributeName, _kSampleCardAttributeCity]);
      expect(mapper.map(_kSampleLocale, input), 'Willeke');

      verify(mockAttributeValueMapper.map(_kSampleLocale, _kSampleCardAttributeName.value)).called(1);
    });

    test('`pid_id` card without `name` attribute should return empty string', () {
      Card input = createSampleCard('pid_id', [_kSampleCardAttributeCity]);
      expect(mapper.map(_kSampleLocale, input), '');

      verifyNever(mockAttributeValueMapper.map(_kSampleLocale, _kSampleCardAttributeName.value));
    });

    test('card with `com.example.address` docType should return `city` attribute string', () {
      when(mockAttributeValueMapper.map(_kSampleLocale, _kSampleCardAttributeCity.value)).thenReturn('Den Haag');

      Card input = createSampleCard('com.example.address', [_kSampleCardAttributeName, _kSampleCardAttributeCity]);
      expect(mapper.map(_kSampleLocale, input), 'Den Haag');

      verify(mockAttributeValueMapper.map(_kSampleLocale, _kSampleCardAttributeCity.value)).called(1);
    });

    test('`pid_address` card with `city` attribute should return `city` attribute string', () {
      when(mockAttributeValueMapper.map(_kSampleLocale, _kSampleCardAttributeCity.value)).thenReturn('Den Haag');

      Card input = createSampleCard('pid_address', [_kSampleCardAttributeName, _kSampleCardAttributeCity]);
      expect(mapper.map(_kSampleLocale, input), 'Den Haag');

      verify(mockAttributeValueMapper.map(_kSampleLocale, _kSampleCardAttributeCity.value)).called(1);
    });

    test('`pid_address` card without `city` attribute should return empty string', () {
      Card input = createSampleCard('pid_address', [_kSampleCardAttributeName]);
      expect(mapper.map(_kSampleLocale, input), '');

      verifyNever(mockAttributeValueMapper.map(_kSampleLocale, _kSampleCardAttributeName.value));
    });

    test('card with unknown docType should return empty string', () {
      Card input = createSampleCard('noneExisting', [_kSampleCardAttributeName, _kSampleCardAttributeCity]);
      expect(mapper.map(_kSampleLocale, input), '');
    });
  });
}
