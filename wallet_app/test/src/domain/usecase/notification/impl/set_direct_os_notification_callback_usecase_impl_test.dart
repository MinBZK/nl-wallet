import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/notification/notification_type.dart';
import 'package:wallet/src/domain/model/notification/os_notification.dart';
import 'package:wallet/src/domain/usecase/notification/impl/set_direct_os_notification_callback_usecase_impl.dart';
import 'package:wallet/src/util/extension/notification_type_extension.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.dart';

void main() {
  late MockNotificationRepository notificationRepository;
  late MockActiveLocaleProvider activeLocaleProvider;
  late SetDirectOsNotificationCallbackUsecaseImpl usecase;

  setUp(() {
    notificationRepository = MockNotificationRepository();
    activeLocaleProvider = MockActiveLocaleProvider();
    usecase = SetDirectOsNotificationCallbackUsecaseImpl(notificationRepository, activeLocaleProvider);

    when(activeLocaleProvider.activeLocale).thenReturn(const Locale('en'));
    when(notificationRepository.arePushNotificationsEnabled()).thenAnswer((_) async => true);
  });

  group('SetDirectOsNotificationCallbackUsecaseImpl', () {
    test('invokes callback when repository triggers notification and setting is enabled', () async {
      final card = WalletMockData.card;
      final type = NotificationType.cardExpired(card: card);
      const id = 123;

      OsNotification? capturedNotification;
      usecase.invoke((notification) {
        capturedNotification = notification;
      });

      // Capture the callback passed to the repository
      final VerificationResult verification = verify(notificationRepository.setDirectNotificationCallback(captureAny));
      final void Function(int, NotificationType) repoCallback = verification.captured.single;

      // Trigger the repository callback
      repoCallback(id, type);
      await Future.delayed(Duration.zero); // Process internal async calls

      // Validate (mapped) output
      expect(capturedNotification, isNotNull);
      expect(capturedNotification!.id, id);
      expect(capturedNotification!.channel, type.channel);
      expect(capturedNotification!.title, type.title(const Locale('en')));
      expect(capturedNotification!.body, type.body(const Locale('en'), capturedNotification!.notifyAt));
    });

    test('does not invoke callback when repository triggers notification and setting is disabled', () async {
      when(notificationRepository.arePushNotificationsEnabled()).thenAnswer((_) async => false);

      final card = WalletMockData.card;
      final type = NotificationType.cardExpired(card: card);
      const id = 123;

      OsNotification? capturedNotification;
      usecase.invoke((notification) {
        capturedNotification = notification;
      });

      // Capture the callback passed to the repository
      final VerificationResult verification = verify(notificationRepository.setDirectNotificationCallback(captureAny));
      final void Function(int, NotificationType) repoCallback = verification.captured.single;

      // Trigger the repository callback
      repoCallback(id, type);
      await Future.delayed(Duration.zero); // Process internal async calls

      // Validate that callback is not invoked
      expect(capturedNotification, isNull);
    });
  });
}
