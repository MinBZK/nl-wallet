import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/util/builder/notification/notification_payload_parser.dart';

void main() {
  group('NotificationPayloadParser', () {
    test('returns null for null payload', () {
      expect(NotificationPayloadParser.parse(null), isNull);
    });

    test('returns null for malformed URI', () {
      expect(NotificationPayloadParser.parse('not a uri'), isNull);
    });

    test('returns null for wrong scheme', () {
      expect(NotificationPayloadParser.parse('https://example.org'), isNull);
    });

    test('parses card detail route correctly', () {
      const payload = 'nlwallet://app/card/detail?id=test-id';
      final result = NotificationPayloadParser.parse(payload);

      expect(result, NavigationRequest.cardDetail('test-id'));
    });

    test('returns null if id is missing in card detail route', () {
      const payload = 'nlwallet://app/card/detail';
      expect(NotificationPayloadParser.parse(payload), isNull);
    });

    test('returns null for unknown route', () {
      const payload = 'nlwallet://app/unknown';
      expect(NotificationPayloadParser.parse(payload), isNull);
    });
  });
}
