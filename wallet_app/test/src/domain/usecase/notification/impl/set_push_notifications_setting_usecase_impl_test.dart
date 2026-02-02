import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/notification/notification_repository.dart';
import 'package:wallet/src/domain/usecase/notification/impl/set_push_notifications_setting_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/notification/set_push_notifications_setting_usecase.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late NotificationRepository mockNotificationRepository;
  late SetPushNotificationsSettingUseCase usecase;

  setUp(() {
    mockNotificationRepository = MockNotificationRepository();
    usecase = SetPushNotificationsSettingUseCaseImpl(mockNotificationRepository);
  });

  group('invoke', () {
    test('should call setShowNotificationsEnabled with true', () async {
      when(
        mockNotificationRepository.setPushNotificationsEnabled(enabled: true),
      ).thenAnswer((_) async => Future.value());

      await usecase.invoke(enabled: true);

      verify(mockNotificationRepository.setPushNotificationsEnabled(enabled: true)).called(1);
      verifyNoMoreInteractions(mockNotificationRepository);
    });

    test('should call setShowNotificationsEnabled with false', () async {
      when(
        mockNotificationRepository.setPushNotificationsEnabled(enabled: false),
      ).thenAnswer((_) async => Future.value());

      await usecase.invoke(enabled: false);

      verify(mockNotificationRepository.setPushNotificationsEnabled(enabled: false)).called(1);
      verifyNoMoreInteractions(mockNotificationRepository);
    });
  });
}
