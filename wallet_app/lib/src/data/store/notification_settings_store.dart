/// Abstract class for managing notification settings.
abstract class NotificationSettingsStore {
  /// Retrieves the flag indicating whether the user should be asked to allow notifications.
  Future<bool?> getShowNotificationRequestFlag();

  /// Sets the flag indicating whether the user should be asked to allow notifications.
  Future<void> setShowNotificationRequestFlag({bool? showNotificationRequest});

  /// Retrieves whether push notifications are enabled.
  Future<bool> getPushNotificationsEnabled();

  /// Enables or disables push notifications.
  Future<void> setPushNotificationsEnabled({required bool enabled});

  /// Observes a stream of the push notifications enabled status.
  Stream<bool> observePushNotificationsEnabled();
}
