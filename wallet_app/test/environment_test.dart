import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/environment.dart';

void main() {
  test('verify default environment values', () {
    expect(Environment.mockRepositories, isTrue);
    expect(Environment.isTest, isTrue);
    expect(Environment.hasSentryDsn, isFalse);
    expect(Environment.sentryRelease(), isNull);
  });
}
