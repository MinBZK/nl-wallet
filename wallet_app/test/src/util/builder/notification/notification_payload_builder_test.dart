import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/notification/app_notification.dart';
import 'package:wallet/src/util/builder/notification/notification_payload_builder.dart';

import '../../../mocks/wallet_mock_data.dart';

void main() {
  group('NotificationPayloadBuilder', () {
    final card = WalletMockData.card.copyWith(attestationId: 'test-id');

    test('builds correct payload for CardExpiresSoon', () {
      final notification = CardExpiresSoon(
        card: card,
        expiresAt: DateTime(2024, 1, 1),
      );

      final result = NotificationPayloadBuilder.build(notification);

      expect(result, 'nlwallet://app/card/detail?id=test-id');
    });

    test('builds correct payload for CardExpired', () {
      final notification = CardExpired(card: card);

      final result = NotificationPayloadBuilder.build(notification);

      expect(result, 'nlwallet://app/card/detail?id=test-id');
    });
  });
}
