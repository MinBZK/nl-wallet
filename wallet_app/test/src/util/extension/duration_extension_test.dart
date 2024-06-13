import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/util/extension/duration_extension.dart';

void main() {
  group('inMonths', () {
    test('90 days duration returns 3', () async {
      expect(const Duration(days: 90).inMonths, 3);
    });

    test('45 days duration returns 1', () async {
      expect(const Duration(days: 45).inMonths, 1);
    });

    test('0 days duration returns 0', () async {
      expect(Duration.zero.inMonths, 0);
    });
  });
}
