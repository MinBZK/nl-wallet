sealed class UpdateNotification {}

class RecommendUpdateNotification extends UpdateNotification {}

class WarnUpdateNotification extends UpdateNotification {
  final Duration timeUntilBlocked;

  WarnUpdateNotification({required this.timeUntilBlocked});
}
