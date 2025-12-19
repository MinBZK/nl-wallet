import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/notification/notification_repository.dart';
import 'package:wallet/src/data/store/active_locale_provider.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/notification/notification_channel.dart';
import 'package:wallet/src/domain/model/notification/os_notification.dart';
import 'package:wallet/src/domain/usecase/notification/impl/observe_os_notifications_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/notification/observe_os_notifications_usecase.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late NotificationRepository mockNotificationRepository;
  late ActiveLocaleProvider mockActiveLocaleProvider;
  late ObserveOsNotificationsUseCase usecase;

  setUp(() async {
    mockNotificationRepository = MockNotificationRepository();
    mockActiveLocaleProvider = MockActiveLocaleProvider();

    when(mockActiveLocaleProvider.activeLocale).thenReturn(const Locale('en'));

    usecase = ObserveOsNotificationsUseCaseImpl(
      mockNotificationRepository,
      mockActiveLocaleProvider,
    );
  });

  group('invoke', () {
    test('should return a stream of OsNotification for OS notifications when enabled', () async {
      final appNotifications = [
        WalletMockData.osCardExpiresSoonNotification,
        WalletMockData.osCardExpiredNotification,
        WalletMockData.dashboardCardExpiresSoonNotification,
      ];

      when(mockNotificationRepository.observeNotifications()).thenAnswer(
        (_) => Stream.value(appNotifications),
      );
      when(mockNotificationRepository.observePushNotificationsEnabled()).thenAnswer(
        (_) => Stream.value(true),
      );

      final expectedOsNotifications = [
        OsNotification(
          id: WalletMockData.osCardExpiresSoonNotification.id,
          channel: NotificationChannel.cardUpdates,
          title: 'NL Wallet',
          body: 'Sample Card #1 expires in 4 days. Replace this card if you still need it.',
          notifyAt: WalletMockData.defaultNotifyAt,
        ),
        OsNotification(
          id: WalletMockData.osCardExpiredNotification.id,
          channel: NotificationChannel.cardUpdates,
          title: 'NL Wallet',
          body: 'Sample Card #1 expired. Replace this card if you still need it.',
          notifyAt: WalletMockData.defaultNotifyAt,
        ),
      ];

      await expectLater(
        usecase.invoke(),
        emitsInOrder([expectedOsNotifications]),
      );

      verify(mockNotificationRepository.observeNotifications()).called(1);
      verify(mockNotificationRepository.observePushNotificationsEnabled()).called(1);
    });

    test('should return an empty list if no OS notifications are present', () async {
      final appNotifications = [
        WalletMockData.dashboardCardExpiresSoonNotification,
      ];

      when(mockNotificationRepository.observeNotifications()).thenAnswer(
        (_) => Stream.value(appNotifications),
      );
      when(mockNotificationRepository.observePushNotificationsEnabled()).thenAnswer(
        (_) => Stream.value(true),
      );

      await expectLater(
        usecase.invoke(),
        emitsInOrder([
          [],
        ]),
      );

      verify(mockNotificationRepository.observeNotifications()).called(1);
      verify(mockNotificationRepository.observePushNotificationsEnabled()).called(1);
      verifyNoMoreInteractions(mockNotificationRepository);
      verifyNoMoreInteractions(mockActiveLocaleProvider);
    });

    test('should return an empty list if notification stream is empty', () async {
      when(mockNotificationRepository.observeNotifications()).thenAnswer(
        (_) => Stream.value([]),
      );
      when(mockNotificationRepository.observePushNotificationsEnabled()).thenAnswer(
        (_) => Stream.value(true),
      );

      await expectLater(
        usecase.invoke(),
        emitsInOrder([
          [],
        ]),
      );

      verify(mockNotificationRepository.observeNotifications()).called(1);
      verify(mockNotificationRepository.observePushNotificationsEnabled()).called(1);
      verifyNoMoreInteractions(mockNotificationRepository);
      verifyNoMoreInteractions(mockActiveLocaleProvider);
    });

    test('should return an empty list when push notifications are disabled', () async {
      final appNotifications = [
        WalletMockData.osCardExpiresSoonNotification,
      ];

      when(mockNotificationRepository.observeNotifications()).thenAnswer(
        (_) => Stream.value(appNotifications),
      );
      when(mockNotificationRepository.observePushNotificationsEnabled()).thenAnswer(
        (_) => Stream.value(false),
      );

      await expectLater(
        usecase.invoke(),
        emitsInOrder([
          [],
        ]),
      );

      verify(mockNotificationRepository.observeNotifications()).called(1);
      verify(mockNotificationRepository.observePushNotificationsEnabled()).called(1);
      verifyNoMoreInteractions(mockNotificationRepository);
      verifyNoMoreInteractions(mockActiveLocaleProvider);
    });
  });
}
