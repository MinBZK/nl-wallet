import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';

const _kGermanLocale = Locale('de');
const _kDutchLocale = Locale('nl');
const _kEnglishLocale = Locale('en');
const _kNonExistingLocale = Locale('noneExisting');

LocalizedText _kSampleCardAttributeLabels = {
  _kGermanLocale: 'Sprache',
  _kEnglishLocale: 'Language',
  _kDutchLocale: 'Taal',
};

void main() {
  group('map', () {
    test('languageCodes should return matching labels', () {
      expect(_kSampleCardAttributeLabels.l10nValueForLocale(_kGermanLocale), 'Sprache');
      expect(_kSampleCardAttributeLabels.l10nValueForLocale(_kDutchLocale), 'Taal');
      expect(_kSampleCardAttributeLabels.l10nValueForLocale(_kEnglishLocale), 'Language');
    });

    test(
        'resolving with a language code that does not exists should default to the english translation if that is available',
        () {
      expect(_kSampleCardAttributeLabels.l10nValueForLocale(_kNonExistingLocale), 'Language');
    });

    test(
        'resolving with a language code that does not exists should default to the first available '
        'translation if neither it, nor an english translation can be found', () {
      expect({_kGermanLocale: 'Sprache', _kDutchLocale: 'Taal'}.l10nValueForLocale(_kNonExistingLocale), 'Sprache');
    });

    test('if no translations are available it should default to an empty string', () {
      expect(<Locale, String>{}.l10nValueForLocale(_kNonExistingLocale), '');
    });

    test('empty labels list should return empty string', () {
      expect(<Locale, String>{}.l10nValueForLocale(_kNonExistingLocale), '');
    });
  });
}
