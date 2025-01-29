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

  late Mapper<AttestationValue, AttributeValue> mapper;

  setUp(() async {
    /// Needed for [DateFormat] to work
    await initializeDateFormatting();

    l10n = await TestUtils.getLocalizations(_kSampleLocale);

    mapper = CardAttributeValueMapper();
  });

  group('map', () {
    test('`AttestationValue_String` should return equal content string', () {
      const AttestationValue input = AttestationValue_String(value: 'NL Wallet');
      final actual = AttributeValueFormatter.formatWithLocale(_kSampleLocale, mapper.map(input));
      expect(actual, 'NL Wallet');
    });

    test('`AttestationValue_Boolean` should return localized `true` string', () {
      const AttestationValue input = AttestationValue_Boolean(value: true);
      final actual = AttributeValueFormatter.formatWithLocale(_kSampleLocale, mapper.map(input));
      expect(actual, l10n.cardValueTrue);
    });

    test('`AttestationValue_Boolean` should return localized `false` string', () {
      const AttestationValue input = AttestationValue_Boolean(value: false);
      final actual = AttributeValueFormatter.formatWithLocale(_kSampleLocale, mapper.map(input));
      expect(actual, l10n.cardValueFalse);
    });
  });
}
