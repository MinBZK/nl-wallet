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
      final serialized = expected.toJson();
      final result = DisclosureScreenArgument.fromJson(serialized);
      expect(result, expected);
    },
  );

  test(
    'hashcode behaves as expected',
    () {
      const a = DisclosureScreenArgument(uri: 'a', isQrCode: true);
      const b = DisclosureScreenArgument(uri: 'a', isQrCode: false);
      expect(a.hashCode, a.hashCode);
      expect(a.hashCode, isNot(b.hashCode));
    },
  );

  test(
    'toString contains uri and isQrCode',
    () {
      const a = DisclosureScreenArgument(uri: 'www.example.org', isQrCode: true);
      expect(a.toString(), contains('www.example.org'));
      expect(a.toString(), contains(true.toString()));
    },
  );
}
