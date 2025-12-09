import 'package:freezed_annotation/freezed_annotation.dart';

import 'notification_display_target.dart';
import 'notification_type.dart';

export 'notification_display_target.dart';
export 'notification_type.dart';

part 'app_notification.freezed.dart';

@freezed
abstract class AppNotification with _$AppNotification {
  const factory AppNotification({
    required int id,
    required NotificationType type,
    required List<NotificationDisplayTarget> displayTargets,
  }) = _AppNotification;
}
