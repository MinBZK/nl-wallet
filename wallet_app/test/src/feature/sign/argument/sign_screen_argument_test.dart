import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/sign/argument/sign_screen_argument.dart';

void main() {
  test(
    'serialize to and from Map<> yields identical object',
    () {
      const expected = SignScreenArgument(
        mockSessionId: '1aef7',
        uri: 'https://example.org',
      );
      final serialized = expected.toJson();
      final result = SignScreenArgument.fromJson(serialized);
      expect(result, expected);
    },
  );
}
