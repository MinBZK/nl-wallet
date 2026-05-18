import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/disclosure/argument/disclosure_screen_argument.dart';
import 'package:wallet/src/feature/disclosure/disclosure_screen.dart';

void main() {
  test(
    'serialize to and from Map<> yields identical object',
    () {
      const expected = DisclosureScreenArgument(
        type: .remote(
          'https://example.org',
          isQrCode: true,
        ),
      );
      final serialized = expected.toJson();
      final result = DisclosureScreenArgument.fromJson(serialized);
      expect(result, expected);
    },
  );

  test(
    'hashcode behaves as expected',
    () {
      const a = DisclosureScreenArgument(type: .remote('a', isQrCode: true));
      const b = DisclosureScreenArgument(type: .remote('a', isQrCode: false));
      expect(a.hashCode, a.hashCode);
      expect(a.hashCode, isNot(b.hashCode));
    },
  );

  test(
    'toString contains uri and isQrCode',
    () {
      const a = DisclosureScreenArgument(type: .remote('www.example.org', isQrCode: true));
      expect(a.toString(), contains('www.example.org'));
      expect(a.toString(), contains(true.toString()));
    },
  );

  group('DisclosureScreen.getArgument', () {
    test('returns DisclosureScreenArgument when passed directly', () {
      const argument = DisclosureScreenArgument(
        type: DisclosureConnectionType.closeProximity(),
      );
      final settings = const RouteSettings(arguments: argument);

      final result = DisclosureScreen.getArgument(settings);

      expect(result, argument);
    });

    test('returns DisclosureScreenArgument when passed as a valid Map (Remote)', () {
      final map = {
        'type': {
          'runtimeType': 'remote',
          'uri': 'https://example.com',
          'isQrCode': true,
        },
      };
      final settings = RouteSettings(arguments: map);

      final result = DisclosureScreen.getArgument(settings);

      expect(result.type, isA<RemoteDisclosure>());
      final remote = result.type as RemoteDisclosure;
      expect(remote.uri, 'https://example.com');
      expect(remote.isQrCode, isTrue);
    });

    test('returns DisclosureScreenArgument when passed as a valid Map (CloseProximity)', () {
      final map = {
        'type': {
          'runtimeType': 'closeProximity',
        },
      };
      final settings = RouteSettings(arguments: map);

      final result = DisclosureScreen.getArgument(settings);

      expect(result.type, isA<CloseProximityDisclosure>());
    });

    test('throws UnsupportedError when arguments are null', () {
      const settings = RouteSettings(arguments: null);

      expect(
        () => DisclosureScreen.getArgument(settings),
        throwsA(isA<UnsupportedError>()),
      );
    });

    test('throws UnsupportedError when arguments are of wrong type', () {
      const settings = RouteSettings(arguments: 'not a map or argument');

      expect(
        () => DisclosureScreen.getArgument(settings),
        throwsA(isA<UnsupportedError>()),
      );
    });

    test('throws UnsupportedError when Map is missing required fields', () {
      final map = {
        'wrong_key': 'wrong_value',
      };
      final settings = RouteSettings(arguments: map);

      expect(
        () => DisclosureScreen.getArgument(settings),
        throwsA(isA<UnsupportedError>()),
      );
    });
  });
}
