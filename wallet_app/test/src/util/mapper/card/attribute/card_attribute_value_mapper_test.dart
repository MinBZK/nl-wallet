import 'dart:ui';

import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:intl/date_symbol_data_local.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/util/formatter/attribute_value_formatter.dart';
import 'package:wallet/src/util/mapper/card/attribute/card_attribute_value_mapper.dart';
import 'package:wallet/src/util/mapper/mapper.dart';
import 'package:wallet_core/core.dart';

import '../../../test_utils.dart';

const _kSampleLocale = Locale('nl');

void main() {
  late AppLocalizations l10n;

  late Mapper<CardValue, AttributeValue> mapper;

  setUp(() async {
    /// Needed for [DateFormat] to work
    await initializeDateFormatting();

    l10n = await TestUtils.getLocalizations(_kSampleLocale);

    mapper = CardAttributeValueMapper();
  });

  group('map', () {
    test('`CardValue_String` should return equal content string', () {
      const CardValue input = CardValue_String(value: 'NL Wallet');
      final actual = AttributeValueFormatter.formatWithLocale(_kSampleLocale, mapper.map(input));
      expect(actual, 'NL Wallet');
    });

    test('`CardValue_Boolean` should return localized `true` string', () {
      const CardValue input = CardValue_Boolean(value: true);
      final actual = AttributeValueFormatter.formatWithLocale(_kSampleLocale, mapper.map(input));
      expect(actual, l10n.cardValueTrue);
    });

    test('`CardValue_Boolean` should return localized `false` string', () {
      const CardValue input = CardValue_Boolean(value: false);
      final actual = AttributeValueFormatter.formatWithLocale(_kSampleLocale, mapper.map(input));
      expect(actual, l10n.cardValueFalse);
    });

    test('`CardValue_Date` should return formatted date string', () {
      const CardValue input = CardValue_Date(value: '2015-10-21');
      final actual = AttributeValueFormatter.formatWithLocale(_kSampleLocale, mapper.map(input));
      expect(actual, '21 oktober 2015');
    });

    test('`CardValue_Gender.NotApplicable` should return localized string', () {
      const CardValue input = CardValue_Gender(value: GenderCardValue.NotApplicable);
      final actual = AttributeValueFormatter.formatWithLocale(_kSampleLocale, mapper.map(input));
      expect(actual, l10n.cardValueGenderNotApplicable);
    });

    test('`CardValue_Gender.Female` should return localized string', () {
      const CardValue input = CardValue_Gender(value: GenderCardValue.Female);
      final actual = AttributeValueFormatter.formatWithLocale(_kSampleLocale, mapper.map(input));
      expect(actual, l10n.cardValueGenderFemale);
    });

    test('`CardValue_Gender.Male` should return localized string', () {
      const CardValue input = CardValue_Gender(value: GenderCardValue.Male);
      final actual = AttributeValueFormatter.formatWithLocale(_kSampleLocale, mapper.map(input));
      expect(actual, l10n.cardValueGenderMale);
    });

    test('`CardValue_Gender.Unknown` should return localized string', () {
      const CardValue input = CardValue_Gender(value: GenderCardValue.Unknown);
      final actual = AttributeValueFormatter.formatWithLocale(_kSampleLocale, mapper.map(input));
      expect(actual, l10n.cardValueGenderUnknown);
    });
  });
}
