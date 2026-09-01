import 'package:flutter_test/flutter_test.dart';
import 'package:intl/date_symbol_data_local.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/util/formatter/attribute_value_formatter.dart';

const _kLocale = Locale('nl');

void main() {
  setUp(() async {
    /// Needed for [DateFormat] to work
    await initializeDateFormatting();
  });

  group('array', () {
    final array = ArrayValue([const StringValue('A'), DateValue(DateTime(2024, 10, 5))]);

    test('is spread over bulleted lines by default', () {
      expect(AttributeValueFormatter.formatWithLocale(_kLocale, array), '  • A\n  • 5-10-2024');
    });

    test('is joined with a comma when inline', () {
      expect(AttributeValueFormatter.formatWithLocale(_kLocale, array, inline: true), 'A, 5-10-2024');
    });
  });

  group('map', () {
    final map = MapValue({
      'vehicle_category_code': const StringValue('B'),
      'issue_date': DateValue(DateTime(2017, 2, 23)),
    });

    test('puts every entry on its own line by default', () {
      expect(
        AttributeValueFormatter.formatWithLocale(_kLocale, map),
        'vehicle_category_code: B\nissue_date: 23-2-2017',
      );
    });

    test('is joined with a comma when inline', () {
      expect(
        AttributeValueFormatter.formatWithLocale(_kLocale, map, inline: true),
        'vehicle_category_code: B, issue_date: 23-2-2017',
      );
    });
  });

  group('nesting', () {
    test('inline propagates into nested values', () {
      const nested = ArrayValue([
        MapValue({'code': StringValue('AM')}),
        MapValue({'code': StringValue('B')}),
      ]);
      expect(AttributeValueFormatter.formatWithLocale(_kLocale, nested, inline: true), 'code: AM, code: B');
    });

    test('an empty nested collection is formatted rather than skipped', () {
      const nested = ArrayValue([ArrayValue([])]);
      final l10n = AttributeValueFormatter.formatWithLocale(_kLocale, const ArrayValue([]));
      expect(AttributeValueFormatter.formatWithLocale(_kLocale, nested, inline: true), l10n);
    });
  });
}
