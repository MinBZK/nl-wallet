import '../../../../data/repository/notification/notification_repository.dart';
import '../../../../data/store/active_locale_provider.dart';
import '../../../../util/builder/notification/notification_payload_builder.dart';
import '../../../../util/extension/notification_type_extension.dart';
import '../../../model/notification/os_notification.dart';
import '../set_direct_os_notification_callback_usecase.dart';

class SetDirectOsNotificationCallbackUsecaseImpl extends SetDirectOsNotificationCallbackUsecase {
  final NotificationRepository _notificationRepository;
  final ActiveLocaleProvider _activeLocaleProvider;

  SetDirectOsNotificationCallbackUsecaseImpl(this._notificationRepository, this._activeLocaleProvider);

  @override
  void invoke(Function(OsNotification) callback) {
    _notificationRepository.setDirectNotificationCallback((id, type) async {
      if (!await _notificationRepository.arePushNotificationsEnabled()) return;
      final notifyAt = DateTime.now();
      callback(
        OsNotification(
          id: id,
          channel: type.channel,
          title: type.title(_activeLocaleProvider.activeLocale),
          body: type.body(_activeLocaleProvider.activeLocale, notifyAt),
          notifyAt: notifyAt,
          payload: NotificationPayloadBuilder.build(type),
        ),
      );
    });
  }
}
