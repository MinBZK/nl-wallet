import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

import 'app_notification.dart';
import 'notification_channel.dart';

part 'os_notification.freezed.dart';

/// Represents the minimal information required to schedule a native OS notification.
///
/// This class is a simpler version of a more complex [AppNotification], containing
/// only the essential details to schedule it using [FlutterLocalNotificationsPlugin].
@freezed
abstract class OsNotification with _$OsNotification {
  const factory OsNotification({
    /// A unique identifier for this notification.
    required int id,

    /// The channel through which this notification will be delivered.
    ///
    /// Channels are used on modern Android versions to group notifications and
    /// allow users to manage them.
    required NotificationChannel channel,

    /// The localized title of the notification.
    required String title,

    /// The localized body of the notification.
    required String body,

    /// The payload of the notification.
    String? payload,

    /// The exact date and time when the notification should be displayed.
    required DateTime notifyAt,
  }) = _OsNotification;
}
