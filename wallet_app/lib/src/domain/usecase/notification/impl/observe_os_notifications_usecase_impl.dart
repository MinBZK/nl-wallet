import 'package:rxdart/rxdart.dart';

import '../../../../data/repository/notification/notification_repository.dart';
import '../../../../data/store/active_locale_provider.dart';
import '../../../../util/builder/notification/notification_payload_builder.dart';
import '../../../../util/extension/notification_type_extension.dart';
import '../../../model/notification/app_notification.dart';
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
  Stream<List<OsNotification>> invoke({bool respectUserSetting = true}) {
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
              channel: osNotification.type.channel,
              title: osNotification.type.title(_activeLocaleProvider.activeLocale),
              body: osNotification.type.body(_activeLocaleProvider.activeLocale, notifyAt),
              notifyAt: notifyAt,
              payload: NotificationPayloadBuilder.build(osNotification.type),
            );
          },
        ).toList();
      },
    );

    if (!respectUserSetting) return notificationStream;

    // Only emit [OsNotification]s when push notification setting is enabled
    return _notificationRepository.observePushNotificationsEnabled().switchMap(
      (enabled) => enabled ? notificationStream : Stream.value([]),
    );
  }
}
