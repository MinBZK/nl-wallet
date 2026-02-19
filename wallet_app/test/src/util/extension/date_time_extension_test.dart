import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/util/extension/date_time_extension.dart';

void main() {
  group('yearMonth', () {
    test('date time returns year/month', () async {
      final dateTime = DateTime.fromMillisecondsSinceEpoch(1685452337965); // 2023-05-30 15:12:17.965 +02:00
      expect(dateTime.yearMonth, DateTime(2023, 5, 1, 0, 0, 0, 0, 0));
    });

    test('year/month only date returns unmodified', () async {
      final dateTime = DateTime.fromMillisecondsSinceEpoch(1682899200000, isUtc: true); // 2023-05-01 00:00:00.000 UTC
      expect(dateTime.yearMonth, dateTime);
    });
  });

  group('isSameDay', () {
    test('returns true for same day but different times', () {
      final morning = DateTime(2023, 5, 30, 8, 0, 0);
      final evening = DateTime(2023, 5, 30, 20, 30, 45);
      expect(morning.isSameDay(evening), isTrue);
    });

    test('returns false for different days', () {
      final day1 = DateTime(2023, 5, 30);
      final day2 = DateTime(2023, 5, 31);
      expect(day1.isSameDay(day2), isFalse);
    });

    test('returns false for different months', () {
      final may = DateTime(2023, 5, 15);
      final june = DateTime(2023, 6, 15);
      expect(may.isSameDay(june), isFalse);
    });

    test('returns false for different years', () {
      final year2023 = DateTime(2023, 5, 30);
      final year2024 = DateTime(2024, 5, 30);
      expect(year2023.isSameDay(year2024), isFalse);
    });
  });
}
