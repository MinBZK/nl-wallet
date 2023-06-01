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
}
