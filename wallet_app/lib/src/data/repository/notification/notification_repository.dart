import '../../../domain/model/notification/app_notification.dart';

export '../../../domain/model/disclosure/start_disclosure_result.dart';

abstract class NotificationRepository {
  Stream<List<AppNotification>> observeNotifications();
}
