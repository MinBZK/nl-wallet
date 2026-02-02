import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/notification/notification_repository.dart';
import 'package:wallet/src/domain/usecase/notification/impl/observe_dashboard_notifications_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/notification/observe_dashboard_notifications_usecase.dart';
import 'package:wallet/src/feature/banner/wallet_banner.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late NotificationRepository mockNotificationRepository;
  late ObserveDashboardNotificationsUseCase usecase;

  setUp(() {
    mockNotificationRepository = MockNotificationRepository();
    usecase = ObserveDashboardNotificationsUseCaseImpl(mockNotificationRepository);
  });

  group('invoke', () {
    test('should return a stream of WalletBanner for dashboard notifications', () async {
      final appNotifications = [
        WalletMockData.dashboardCardExpiresSoonNotification,
        WalletMockData.dashboardCardExpiredNotification,
        WalletMockData.osCardExpiresSoonNotification,
      ];

      when(mockNotificationRepository.observeNotifications()).thenAnswer(
        (_) => Stream.value(appNotifications),
      );

      final expectedBanners = [
        CardExpiresSoonBanner(
          card: WalletMockData.dashboardCardExpiresSoonNotification.type.card,
          expiresAt: WalletMockData.expiresIn30Days,
        ),
        CardExpiredBanner(card: WalletMockData.card),
      ];

      await expectLater(
        usecase.invoke(),
        emitsInOrder([
          expectedBanners,
        ]),
      );

      verify(mockNotificationRepository.observeNotifications()).called(1);
      verifyNoMoreInteractions(mockNotificationRepository);
    });

    test('should return an empty list if no dashboard notifications are present', () async {
      final appNotifications = [
        WalletMockData.osCardExpiresSoonNotification,
        WalletMockData.osCardExpiredNotification,
      ];

      when(mockNotificationRepository.observeNotifications()).thenAnswer((_) => Stream.value(appNotifications));

      await expectLater(
        usecase.invoke(),
        emitsInOrder([
          [],
        ]),
      );

      verify(mockNotificationRepository.observeNotifications()).called(1);
      verifyNoMoreInteractions(mockNotificationRepository);
    });

    test('should return an empty list if notification stream is empty', () async {
      when(mockNotificationRepository.observeNotifications()).thenAnswer(
        (_) => Stream.value([]),
      );

      await expectLater(
        usecase.invoke(),
        emitsInOrder([
          [],
        ]),
      );

      verify(mockNotificationRepository.observeNotifications()).called(1);
      verifyNoMoreInteractions(mockNotificationRepository);
    });
  });
}
