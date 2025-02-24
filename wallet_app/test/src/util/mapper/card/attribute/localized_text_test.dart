import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';

const _kGermanLocale = Locale('de');
const _kDutchLocale = Locale('nl');
const _kEnglishLocale = Locale('en');
const _kNonExistingLocale = Locale('noneExisting');

const _kSampleCardAttributeLabels = {'de': 'Sprache', 'en': 'Language', 'nl': 'Taal'};

void main() {
  group('map', () {
    test('languageCodes should return matching labels', () {
      expect(_kSampleCardAttributeLabels.l10nValueForLanguageCode(_kGermanLocale.languageCode), 'Sprache');
      expect(_kSampleCardAttributeLabels.l10nValueForLanguageCode(_kDutchLocale.languageCode), 'Taal');
      expect(_kSampleCardAttributeLabels.l10nValueForLanguageCode(_kEnglishLocale.languageCode), 'Language');
    });

    test(
        'resolving with a language code that does not exists should default to the english translation if that is available',
        () {
      expect(_kSampleCardAttributeLabels.l10nValueForLanguageCode(_kNonExistingLocale.languageCode), 'Language');
    });

    test(
        'resolving with a language code that does not exists should default to the first available '
        'translation if neither it, nor an english translation can be found', () {
      expect({'de': 'Sprache', 'nl': 'Taal'}.l10nValueForLanguageCode(_kNonExistingLocale.languageCode), 'Sprache');
    });

    test('if no translations are available it should default to an empty string', () {
      expect(<String, String>{}.l10nValueForLanguageCode(_kNonExistingLocale.languageCode), '');
    });

    test('empty labels list should return empty string', () {
      expect(<String, String>{}.l10nValueForLanguageCode(_kNonExistingLocale.languageCode), '');
    });
  });
}
