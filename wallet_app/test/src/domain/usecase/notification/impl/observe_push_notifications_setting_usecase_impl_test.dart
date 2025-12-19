import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/notification/notification_repository.dart';
import 'package:wallet/src/domain/usecase/notification/impl/observe_push_notifications_setting_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/notification/observe_push_notifications_setting_usecase.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late NotificationRepository mockNotificationRepository;
  late ObservePushNotificationsSettingUseCase usecase;

  setUp(() {
    mockNotificationRepository = MockNotificationRepository();
    usecase = ObservePushNotificationsSettingUseCaseImpl(mockNotificationRepository);
  });

  group('invoke', () {
    test('should return the stream from notification repository', () async {
      when(mockNotificationRepository.observePushNotificationsEnabled()).thenAnswer(
        (_) => (() async* {
          yield true;
          await Future.delayed(const Duration(milliseconds: 10));
          yield false;
          await Future.delayed(const Duration(milliseconds: 10));
          yield true;
        })(),
      );

      await expectLater(usecase.invoke(), emitsInOrder([true, false, true]));
      verify(mockNotificationRepository.observePushNotificationsEnabled()).called(1);
      verifyNoMoreInteractions(mockNotificationRepository);
    });
  });
}
