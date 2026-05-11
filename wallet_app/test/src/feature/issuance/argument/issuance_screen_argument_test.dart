import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/issuance/argument/issuance_screen_argument.dart';
import 'package:wallet/src/feature/issuance/issuance_screen.dart';

void main() {
  group('IssuanceScreenArgument', () {
    test(
      'ltc5 serialize to and from Map<> yields identical object',
      () {
        const expected = IssuanceScreenArgument(
          mockSessionId: '1aef7',
          isRefreshFlow: true,
          uri: 'https://example.org',
          isQrCode: false,
        );
        final serialized = expected.toJson();
        final result = IssuanceScreenArgument.fromJson(serialized);
        expect(result, expected);
      },
    );
  });

  group('IssuanceScreen.getArgument', () {
    test('returns argument when passed directly', () {
      const expected = IssuanceScreenArgument(isQrCode: true, mockSessionId: '123');
      const settings = RouteSettings(arguments: expected);
      final result = IssuanceScreen.getArgument(settings);
      expect(result, expected);
    });

    test('returns argument when passed as json map', () {
      const expected = IssuanceScreenArgument(isQrCode: true, mockSessionId: '123');
      final settings = RouteSettings(arguments: expected.toJson());
      final result = IssuanceScreen.getArgument(settings);
      expect(result, expected);
    });

    test('throws UnsupportedError when arguments is null', () {
      const settings = RouteSettings(arguments: null);
      expect(() => IssuanceScreen.getArgument(settings), throwsA(isA<UnsupportedError>()));
    });

    test('throws UnsupportedError when arguments is of wrong type', () {
      const settings = RouteSettings(arguments: 'wrong type');
      expect(() => IssuanceScreen.getArgument(settings), throwsA(isA<UnsupportedError>()));
    });
  });
}
