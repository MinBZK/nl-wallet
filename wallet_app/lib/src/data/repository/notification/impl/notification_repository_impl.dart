import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/notification/app_notification.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../../../store/notification_settings_store.dart';
import '../notification_repository.dart';

class NotificationRepositoryImpl implements NotificationRepository {
  final TypedWalletCore _core;
  final Mapper<core.AppNotification, AppNotification> _notificationMapper;
  final NotificationSettingsStore _notificationSettingsStore;

  NotificationRepositoryImpl(
    this._core,
    this._notificationMapper,
    this._notificationSettingsStore,
  );

  @override
  Stream<List<AppNotification>> observeNotifications() => _core.observeNotifications().map(_notificationMapper.mapList);

  @override
  Future<bool?> getShowNotificationRequestFlag() => _notificationSettingsStore.getShowNotificationRequestFlag();

  @override
  Future<void> setShowNotificationRequestFlag({bool? showNotificationRequest}) =>
      _notificationSettingsStore.setShowNotificationRequestFlag(showNotificationRequest: showNotificationRequest);

  @override
  Future<void> setPushNotificationsEnabled({required bool enabled}) =>
      _notificationSettingsStore.setPushNotificationsEnabled(enabled: enabled);

  @override
  Stream<bool> observePushNotificationsEnabled() => _notificationSettingsStore.observePushNotificationsEnabled();
}
