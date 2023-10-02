import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/bridge_generated.dart';
import 'package:wallet/src/util/mapper/card/attribute/card_attribute_label_mapper.dart';
import 'package:wallet/src/util/mapper/locale_mapper.dart';

const _kGermanLocale = Locale('de');
const _kDutchLocale = Locale('nl');
const _kEnglishLocale = Locale('en');
const _kNonExistingLocale = Locale('noneExisting');

const _kSampleCardAttributeLabels = [
  LocalizedString(language: 'de', value: 'Sprache'),
  LocalizedString(language: 'en', value: 'Language'),
  LocalizedString(language: 'nl', value: 'Taal'),
];

void main() {
  late LocaleMapper<List<LocalizedString>, String> mapper;

  setUp(() {
    mapper = CardAttributeLabelMapper();
  });

  group('map', () {
    test('languageCodes should return matching labels', () {
      expect(mapper.map(_kGermanLocale, _kSampleCardAttributeLabels), 'Sprache');
      expect(mapper.map(_kDutchLocale, _kSampleCardAttributeLabels), 'Taal');
      expect(mapper.map(_kEnglishLocale, _kSampleCardAttributeLabels), 'Language');
    });

    test('languageCode `noneExisting` should return first label from list', () {
      expect(mapper.map(_kNonExistingLocale, _kSampleCardAttributeLabels), 'Sprache');
    });

    test('empty labels list should return empty string', () {
      expect(mapper.map(_kNonExistingLocale, []), '');
    });
  });
}
