import 'package:freezed_annotation/freezed_annotation.dart';

part 'notification_display_target.freezed.dart';

@freezed
sealed class NotificationDisplayTarget with _$NotificationDisplayTarget {
  const factory NotificationDisplayTarget.os({
    required DateTime notifyAt,
  }) = Os;

  const factory NotificationDisplayTarget.dashboard() = Dashboard;
}
