abstract class NotificationSettingsStore {
  Future<bool?> getShowNotificationRequest();

  Future<void> setShowNotificationRequest({bool? showNotificationRequest});
}
