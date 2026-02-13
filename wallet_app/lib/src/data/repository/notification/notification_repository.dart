import '../../../domain/model/notification/app_notification.dart';

export '../../../domain/model/disclosure/start_disclosure_result.dart';

/// Abstract class for managing notification-related data and operations.
abstract class NotificationRepository {
  /// Observes a stream of [AppNotification]s.
  Stream<List<AppNotification>> observeNotifications();

  /// Retrieves the flag indicating whether the user should be asked to allow notifications.
  Future<bool?> getShowNotificationRequestFlag();

  /// Sets the flag indicating whether the user should be asked to allow notifications.
  Future<void> setShowNotificationRequestFlag({bool? showNotificationRequest});

  /// Enables or disables push notifications (in-app setting).
  Future<void> setPushNotificationsEnabled({required bool enabled});

  /// Observes a stream of the push notifications enabled status.
  Stream<bool> observePushNotificationsEnabled();

  /// Fetches the current push notifications enabled status.
  Future<bool> arePushNotificationsEnabled();

  /// Registers a callback to be invoked immediately when a notification is received
  ///
  /// The [callback] receives:
  /// - [int] id: The unique identifier of the notification.
  /// - [NotificationType] type: The specific domain model representing the notification payload.
  void setDirectNotificationCallback(Function(int, NotificationType) callback);
}
