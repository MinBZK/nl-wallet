import 'dart:math' as math;

import '../../domain/model/attribute/attribute.dart';
import '../../domain/model/notification/app_notification.dart';
import '../../domain/model/notification/notification_channel.dart';
import 'locale_extension.dart';

extension NotificationTypeExtension on NotificationType {
  NotificationChannel get channel => switch (this) {
    CardExpiresSoon() => NotificationChannel.cardUpdates,
    CardExpired() => NotificationChannel.cardUpdates,
    CardRevoked() => NotificationChannel.cardUpdates,
  };

  String title(Locale locale) => switch (this) {
    CardExpiresSoon() => locale.l10n.cardExpiresSoonNotificationTitle,
    CardExpired() => locale.l10n.cardExpiredNotificationTitle,
    CardRevoked() => locale.l10n.cardRevokedNotificationTitle,
  };

  String body(Locale locale, DateTime notifyAt) {
    final localizedCardTitle = card.title.l10nValueForLocale(locale);
    return switch (this) {
      CardExpiresSoon(:final expiresAt) => locale.l10n.cardExpiresSoonNotificationDescription(
        localizedCardTitle,
        math.max(expiresAt.difference(notifyAt).inDays, 0),
      ),
      CardExpired() => locale.l10n.cardExpiredNotificationDescription(localizedCardTitle),
      CardRevoked() => locale.l10n.cardRevokedNotificationDescription(localizedCardTitle),
    };
  }
}
