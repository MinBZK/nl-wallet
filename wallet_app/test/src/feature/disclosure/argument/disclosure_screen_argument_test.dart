import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/disclosure/argument/disclosure_screen_argument.dart';

void main() {
  test(
    'serialize to and from Map<> yields identical object',
    () {
      const expected = DisclosureScreenArgument(
        uri: 'https://example.org',
        isQrCode: true,
      );
      final serialized = expected.toMap();
      final result = DisclosureScreenArgument.fromMap(serialized);
      expect(result, expected);
    },
  );
}
