import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:intl/date_symbol_data_local.dart';
import 'package:wallet/bridge_generated.dart';
import 'package:wallet/src/util/mapper/card/attribute/card_attribute_value_mapper.dart';
import 'package:wallet/src/util/mapper/locale_mapper.dart';

import '../../../test_utils.dart';

const _kSampleLocale = Locale('nl');

void main() {
  late AppLocalizations l10n;

  late LocaleMapper<CardValue, String> mapper;

  setUp(() async {
    /// Needed for [DateFormat] to work
    initializeDateFormatting();

    l10n = await TestUtils.getLocalizations(_kSampleLocale);

    mapper = CardAttributeValueMapper();
  });

  group('map', () {
    test('`CardValue_String` should return equal content string', () {
      CardValue input = const CardValue_String(value: 'NL Wallet');
      expect(mapper.map(_kSampleLocale, input), 'NL Wallet');
    });

    test('`CardValue_Boolean` should return localized `true` string', () {
      CardValue input = const CardValue_Boolean(value: true);
      expect(mapper.map(_kSampleLocale, input), l10n.cardValueTrue);
    });

    test('`CardValue_Boolean` should return localized `false` string', () {
      CardValue input = const CardValue_Boolean(value: false);
      expect(mapper.map(_kSampleLocale, input), l10n.cardValueFalse);
    });

    test('`CardValue_Date` should return formatted date string', () {
      CardValue input = const CardValue_Date(value: '2015-10-21');
      expect(mapper.map(_kSampleLocale, input), '21 oktober 2015');
    });

    test('`CardValue_Gender.NotApplicable` should return localized string', () {
      CardValue input = const CardValue_Gender(value: GenderCardValue.NotApplicable);
      expect(mapper.map(_kSampleLocale, input), l10n.cardValueGenderNotApplicable);
    });

    test('`CardValue_Gender.Female` should return localized string', () {
      CardValue input = const CardValue_Gender(value: GenderCardValue.Female);
      expect(mapper.map(_kSampleLocale, input), l10n.cardValueGenderFemale);
    });

    test('`CardValue_Gender.Male` should return localized string', () {
      CardValue input = const CardValue_Gender(value: GenderCardValue.Male);
      expect(mapper.map(_kSampleLocale, input), l10n.cardValueGenderMale);
    });

    test('`CardValue_Gender.Unknown` should return localized string', () {
      CardValue input = const CardValue_Gender(value: GenderCardValue.Unknown);
      expect(mapper.map(_kSampleLocale, input), l10n.cardValueGenderUnknown);
    });
  });
}
