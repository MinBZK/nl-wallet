import '../../../../data/repository/notification/notification_repository.dart';
import '../set_push_notifications_setting_usecase.dart';

class SetPushNotificationsSettingUseCaseImpl extends SetPushNotificationsSettingUseCase {
  final NotificationRepository _notificationRepository;

  SetPushNotificationsSettingUseCaseImpl(this._notificationRepository);

  @override
  Future<void> invoke({required bool enabled}) async =>
      _notificationRepository.setPushNotificationsEnabled(enabled: enabled);
}
