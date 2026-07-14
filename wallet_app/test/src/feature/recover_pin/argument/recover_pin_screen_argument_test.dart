import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/recover_pin/argument/recover_pin_screen_argument.dart';

void main() {
  test(
    'serialize to and from Map<> yields identical object',
    () {
      const expected = RecoverPinScreenArgument(uri: 'http://example.org', isRecoveryFlow: true);
      final serialized = expected.toJson();
      final result = RecoverPinScreenArgument.fromJson(serialized);
      expect(result, expected);
    },
  );

  test(
    'hashCode matches on identical objects',
    () {
      const a = RecoverPinScreenArgument(uri: 'http://example.org', isRecoveryFlow: true);
      const b = RecoverPinScreenArgument(uri: 'http://example.org', isRecoveryFlow: true);
      expect(a.hashCode, b.hashCode);
    },
  );

  test(
    'hashCode differs on non identical objects',
    () {
      const a = RecoverPinScreenArgument(uri: 'http://example.org', isRecoveryFlow: true);
      const b = RecoverPinScreenArgument(uri: 'http://example.org', isRecoveryFlow: false);
      const c = RecoverPinScreenArgument(uri: 'http://example.org/another', isRecoveryFlow: true);
      expect(a.hashCode, isNot(b.hashCode));
      expect(a.hashCode, isNot(c.hashCode));
    },
  );
}
