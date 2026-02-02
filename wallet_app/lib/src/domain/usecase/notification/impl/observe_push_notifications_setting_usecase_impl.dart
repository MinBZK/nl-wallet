import '../../../../data/repository/notification/notification_repository.dart';
import '../observe_push_notifications_setting_usecase.dart';

class ObservePushNotificationsSettingUseCaseImpl extends ObservePushNotificationsSettingUseCase {
  final NotificationRepository _notificationRepository;

  ObservePushNotificationsSettingUseCaseImpl(this._notificationRepository);

  @override
  Stream<bool> invoke() => _notificationRepository.observePushNotificationsEnabled();
}
