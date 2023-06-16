import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/pin_timeout/argument/pin_timeout_screen_argument.dart';

void main() {
  test(
    'serialize to and from Map<> yields identical object',
    () {
      DateTime dateWithSeconds = DateTime(2023, 12, 1, 23, 59, 59);
      final expected = PinTimeoutScreenArgument(expiryTime: dateWithSeconds);
      final serialized = expected.toMap();
      final result = PinTimeoutScreenArgument.fromMap(serialized);
      expect(result, expected);
    },
  );
}
