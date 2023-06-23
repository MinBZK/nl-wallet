import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/issuance/argument/issuance_screen_argument.dart';

void main() {
  test(
    'serialize to and from Map<> yields identical object',
    () {
      const expected = IssuanceScreenArgument(sessionId: '1aef7', isRefreshFlow: true);
      final serialized = expected.toMap();
      final result = IssuanceScreenArgument.fromMap(serialized);
      expect(result, expected);
    },
  );
}
