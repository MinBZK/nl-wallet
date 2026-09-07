import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:intl/date_symbol_data_local.dart';
import 'package:wallet/l10n/generated/app_localizations.dart';
import 'package:wallet/src/domain/model/app_image_data.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/util/formatter/attribute_value_formatter.dart';
import 'package:wallet/src/util/mapper/card/attribute/card_attribute_value_mapper.dart';
import 'package:wallet/src/util/mapper/image/image_mapper.dart';
import 'package:wallet/src/util/mapper/mapper.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../../test_util/test_utils.dart';

const _kSampleLocale = Locale('nl');

void main() {
  late AppLocalizations l10n;

  late Mapper<core.AttributeValue, AttributeValue> mapper;

  setUp(() async {
    /// Needed for [DateFormat] to work
    await initializeDateFormatting();

    l10n = await TestUtils.getLocalizations(_kSampleLocale);

    mapper = CardAttributeValueMapper(ImageMapper());
  });

  group('map', () {
    test('`AttributeValue_String` should return equal content string', () {
      const core.AttributeValue input = core.AttributeValue_String(value: 'NL Wallet');
      final actual = AttributeValueFormatter.formatWithLocale(_kSampleLocale, mapper.map(input));
      expect(actual, 'NL Wallet');
    });

    test('`AttributeValue_Boolean` should return localized `true` string', () {
      const core.AttributeValue input = core.AttributeValue_Boolean(value: true);
      final actual = AttributeValueFormatter.formatWithLocale(_kSampleLocale, mapper.map(input));
      expect(actual, l10n.cardValueTrue);
    });

    test('`AttributeValue_Boolean` should return localized `false` string', () {
      const core.AttributeValue input = core.AttributeValue_Boolean(value: false);
      final actual = AttributeValueFormatter.formatWithLocale(_kSampleLocale, mapper.map(input));
      expect(actual, l10n.cardValueFalse);
    });

    test('`AttributeValue_Number` should be mapped into a `NumberValue`', () {
      const core.AttributeValue input = core.AttributeValue_Number(value: 1337);
      expect(mapper.map(input), const NumberValue(1337));
      expect(AttributeValueFormatter.formatWithLocale(_kSampleLocale, mapper.map(input)), '1337');
    });

    test('`AttributeValue_Date` should be parsed into a `DateValue`', () {
      const core.AttributeValue input = core.AttributeValue_Date(value: '2030-07-01');
      expect(mapper.map(input), DateValue(DateTime(2030, 7, 1)));
    });

    test('`AttributeValue_Image` should be mapped through the `ImageMapper`', () {
      final data = Uint8List.fromList([1, 2, 3]);
      final core.AttributeValue input = core.AttributeValue_Image(value: core.Image_Png(data: data));
      expect(mapper.map(input), ImageValue(AppMemoryImage(data)));
    });

    test('`AttributeValue_Bytes` should be kept as bytes rather than stringified', () {
      final data = Uint8List.fromList([1, 2, 3]);
      final core.AttributeValue input = core.AttributeValue_Bytes(value: data);
      expect(mapper.map(input), BytesValue(data));
    });

    test('`AttributeValue_Map` should keep keys and their order', () {
      const core.AttributeValue input = core.AttributeValue_Map(
        value: [
          ('vehicle_category_code', core.AttributeValue_String(value: 'B')),
          ('issue_date', core.AttributeValue_Date(value: '2017-02-23')),
        ],
      );
      final actual = mapper.map(input) as MapValue;
      expect(actual.value.keys, ['vehicle_category_code', 'issue_date']);
      expect(actual.value['vehicle_category_code'], const StringValue('B'));
      expect(actual.value['issue_date'], DateValue(DateTime(2017, 2, 23)));
    });
  });
}
