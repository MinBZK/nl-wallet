import 'dart:math' as math;

import 'package:rxdart/rxdart.dart';

import '../../../../data/repository/notification/notification_repository.dart';
import '../../../../data/store/active_locale_provider.dart';
import '../../../../util/extension/locale_extension.dart';
import '../../../model/attribute/attribute.dart';
import '../../../model/notification/app_notification.dart';
import '../../../model/notification/notification_channel.dart';
import '../../../model/notification/os_notification.dart';
import '../observe_os_notifications_usecase.dart';

class ObserveOsNotificationsUseCaseImpl extends ObserveOsNotificationsUseCase {
  final NotificationRepository _notificationRepository;
  final ActiveLocaleProvider _activeLocaleProvider;

  ObserveOsNotificationsUseCaseImpl(
    this._notificationRepository,
    this._activeLocaleProvider,
  );

  @override
  Stream<List<OsNotification>> invoke() {
    final notificationStream = _notificationRepository.observeNotifications().map(
      (input) {
        // Filter out notifications which target the [Os].
        final osNotifications = input.where((it) => it.displayTargets.any((target) => target is Os)).toList();
        // Map them to simple [OsNotification], ready to be scheduled
        return osNotifications.map(
          (osNotification) {
            final notifyAt = osNotification.displayTargets.whereType<Os>().first.notifyAt;
            return OsNotification(
              id: osNotification.id,
              channel: _resolveChannel(osNotification.type),
              title: _resolveTitle(osNotification.type),
              body: _resolveBody(osNotification.type, notifyAt),
              notifyAt: notifyAt,
            );
          },
        ).toList();
      },
    );

    // Only emit [OsNotification]s when push notification setting is enabled
    return _notificationRepository.observePushNotificationsEnabled().switchMap(
      (enabled) => enabled ? notificationStream : Stream.value([]),
    );
  }

  NotificationChannel _resolveChannel(NotificationType type) {
    return switch (type) {
      CardExpiresSoon() => .cardUpdates,
      CardExpired() => .cardUpdates,
    };
  }

  String _resolveTitle(NotificationType type) {
    final l10n = _activeLocaleProvider.activeLocale.l10n;
    return switch (type) {
      CardExpiresSoon() => l10n.cardExpiresSoonNotificationTitle,
      CardExpired() => l10n.cardExpiredNotificationTitle,
    };
  }

  String _resolveBody(NotificationType type, DateTime notifyAt) {
    final l10n = _activeLocaleProvider.activeLocale.l10n;
    return switch (type) {
      CardExpiresSoon() => l10n.cardExpiresSoonNotificationDescription(
        type.card.title.l10nValueForLocale(_activeLocaleProvider.activeLocale),
        math.max(type.expiresAt.difference(notifyAt).inDays, 0 /* fallback value */),
      ),
      CardExpired() => l10n.cardExpiredNotificationDescription(
        type.card.title.l10nValueForLocale(_activeLocaleProvider.activeLocale),
      ),
    };
  }
}
