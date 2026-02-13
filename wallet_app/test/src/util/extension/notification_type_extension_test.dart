import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/notification/notification_channel.dart';
import 'package:wallet/src/domain/model/notification/notification_type.dart';
import 'package:wallet/src/util/extension/notification_type_extension.dart';

import '../../mocks/wallet_mock_data.dart';

void main() {
  group('NotificationTypeExtension', () {
    const locale = Locale('en');
    final card = WalletMockData.card;

    group('channel', () {
      test('CardExpiresSoon returns cardUpdates', () {
        final notification = NotificationType.cardExpiresSoon(
          card: card,
          expiresAt: DateTime(2024, 1, 1),
        );
        expect(notification.channel, NotificationChannel.cardUpdates);
      });

      test('CardExpired returns cardUpdates', () {
        final notification = NotificationType.cardExpired(card: card);
        expect(notification.channel, NotificationChannel.cardUpdates);
      });

      test('CardRevoked returns cardUpdates', () {
        final notification = NotificationType.cardRevoked(card: card);
        expect(notification.channel, NotificationChannel.cardUpdates);
      });
    });

    group('title', () {
      test('CardExpiresSoon returns correct title', () {
        final notification = NotificationType.cardExpiresSoon(
          card: card,
          expiresAt: DateTime(2024, 1, 1),
        );
        expect(notification.title(locale), 'NL Wallet');
      });

      test('CardExpired returns correct title', () {
        final notification = NotificationType.cardExpired(card: card);
        expect(notification.title(locale), 'NL Wallet');
      });

      test('CardRevoked returns correct title', () {
        final notification = NotificationType.cardRevoked(card: card);
        expect(notification.title(locale), 'NL Wallet');
      });
    });

    group('body', () {
      test('CardExpiresSoon returns correct body with days left', () {
        final expiresAt = DateTime(2024, 1, 10);
        final notifyAt = DateTime(2024, 1, 1);
        final notification = NotificationType.cardExpiresSoon(
          card: card,
          expiresAt: expiresAt,
        );

        expect(
          notification.body(locale, notifyAt),
          'Sample Card #1 expires in 9 days. Replace this card if you still need it.',
        );
      });

      test('CardExpired returns correct body', () {
        final notification = NotificationType.cardExpired(card: card);
        expect(
          notification.body(locale, DateTime.now()),
          'Sample Card #1 expired. Replace this card if you still need it.',
        );
      });

      test('CardRevoked returns correct body', () {
        final notification = NotificationType.cardRevoked(card: card);
        expect(
          notification.body(locale, DateTime.now()),
          'Sample Card #1 withdrawn by issuer. Replace this card if you still need it.',
        );
      });
    });
  });
}
