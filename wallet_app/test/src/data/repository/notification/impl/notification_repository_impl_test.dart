import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/notification/impl/notification_repository_impl.dart';
import 'package:wallet/src/data/repository/notification/notification_repository.dart';
import 'package:wallet/src/domain/model/notification/app_notification.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../../mocks/core_mock_data.dart';
import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockTypedWalletCore mockCore;
  late MockMapper<core.AppNotification, AppNotification> mockNotificationMapper;
  late MockMapper<core.NotificationType, NotificationType> mockNotificationTypeMapper;
  late MockNotificationSettingsStore mockNotificationSettingsStore;
  late NotificationRepository notificationRepository;

  setUp(() {
    mockCore = MockTypedWalletCore();
    mockNotificationMapper = MockMapper<core.AppNotification, AppNotification>();
    mockNotificationTypeMapper = MockMapper<core.NotificationType, NotificationType>();
    mockNotificationSettingsStore = MockNotificationSettingsStore();
    notificationRepository = NotificationRepositoryImpl(
      mockCore,
      mockNotificationMapper,
      mockNotificationTypeMapper,
      mockNotificationSettingsStore,
    );
  });

  group('NotificationRepositoryImpl', () {
    group('observeNotifications', () {
      test('should call core.observeNotifications and map the results', () async {
        final coreNotifications = [
          const core.AppNotification(
            id: 4,
            targets: [core.DisplayTarget.dashboard()],
            typ: core.NotificationType_CardExpired(card: CoreMockData.attestation),
          ),
        ];
        final appNotifications = [WalletMockData.dashboardCardExpiredNotification];

        when(mockCore.observeNotifications()).thenAnswer((_) => Stream.value(coreNotifications));
        when(mockNotificationMapper.mapList(coreNotifications)).thenReturn(appNotifications);

        await expectLater(notificationRepository.observeNotifications(), emits(appNotifications));

        verify(mockCore.observeNotifications()).called(1);
        verify(mockNotificationMapper.mapList(coreNotifications)).called(1);
        verifyNoMoreInteractions(mockCore);
        verifyNoMoreInteractions(mockNotificationMapper);
      });
    });

    group('getShowNotificationRequestFlag', () {
      test('should call notificationSettingsStore.getShowNotificationRequestFlag', () async {
        const expectedFlag = true;
        when(mockNotificationSettingsStore.getShowNotificationRequestFlag()).thenAnswer((_) async => expectedFlag);

        final result = await notificationRepository.getShowNotificationRequestFlag();

        expect(result, expectedFlag);
        verify(mockNotificationSettingsStore.getShowNotificationRequestFlag()).called(1);
        verifyNoMoreInteractions(mockNotificationSettingsStore);
      });

      test('should return null if flag is not set', () async {
        when(mockNotificationSettingsStore.getShowNotificationRequestFlag()).thenAnswer((_) async => null);

        final result = await notificationRepository.getShowNotificationRequestFlag();

        expect(result, isNull);
        verify(mockNotificationSettingsStore.getShowNotificationRequestFlag()).called(1);
        verifyNoMoreInteractions(mockNotificationSettingsStore);
      });
    });

    group('setShowNotificationRequestFlag', () {
      test('should call notificationSettingsStore.setShowNotificationRequestFlag with the provided flag', () async {
        const flagToSet = false;
        when(
          mockNotificationSettingsStore.setShowNotificationRequestFlag(
            showNotificationRequest: anyNamed('showNotificationRequest'),
          ),
        ).thenAnswer((_) async => Future.value());

        await notificationRepository.setShowNotificationRequestFlag(showNotificationRequest: flagToSet);

        verify(
          mockNotificationSettingsStore.setShowNotificationRequestFlag(showNotificationRequest: flagToSet),
        ).called(1);
        verifyNoMoreInteractions(mockNotificationSettingsStore);
      });

      test(
        'should call notificationSettingsStore.setShowNotificationRequestFlag with null when null is provided',
        () async {
          when(
            mockNotificationSettingsStore.setShowNotificationRequestFlag(showNotificationRequest: null),
          ).thenAnswer((_) async => Future.value());

          await notificationRepository.setShowNotificationRequestFlag(showNotificationRequest: null);

          verify(mockNotificationSettingsStore.setShowNotificationRequestFlag(showNotificationRequest: null)).called(1);
          verifyNoMoreInteractions(mockNotificationSettingsStore);
        },
      );
    });

    group('setPushNotificationsEnabled', () {
      test('should call notificationSettingsStore.setPushNotificationsEnabled with the provided value', () async {
        const enabled = true;
        when(
          mockNotificationSettingsStore.setPushNotificationsEnabled(enabled: anyNamed('enabled')),
        ).thenAnswer((_) async => Future.value());

        await notificationRepository.setPushNotificationsEnabled(enabled: enabled);

        verify(mockNotificationSettingsStore.setPushNotificationsEnabled(enabled: enabled)).called(1);
        verifyNoMoreInteractions(mockNotificationSettingsStore);
      });
    });

    group('observePushNotificationsEnabled', () {
      test('should call notificationSettingsStore.observePushNotificationsEnabled', () {
        const enabledStatus = false;
        when(
          mockNotificationSettingsStore.observePushNotificationsEnabled(),
        ).thenAnswer((_) => Stream.value(enabledStatus));

        expect(notificationRepository.observePushNotificationsEnabled(), emits(enabledStatus));

        verify(mockNotificationSettingsStore.observePushNotificationsEnabled()).called(1);
        verifyNoMoreInteractions(mockNotificationSettingsStore);
      });
    });

    group('setDirectNotificationCallback', () {
      test('should call core.setupNotificationCallback and map the results when triggered', () {
        // Arrange
        const int notificationId = 1;
        const coreType = core.NotificationType_CardExpired(card: CoreMockData.attestation);
        final appType = NotificationType.cardExpired(card: WalletMockData.card);

        int? capturedId;
        NotificationType? capturedType;

        // Act
        notificationRepository.setDirectNotificationCallback((id, type) {
          capturedId = id;
          capturedType = type;
        });

        // Verify the repository registered the callback with core
        final captured = verify(mockCore.setupNotificationCallback(captureAny)).captured;
        final capturedCallback = captured.first as void Function(List<(int, core.NotificationType)>);

        // Mock the mapper's behavior for when the callback is triggered
        when(mockNotificationTypeMapper.map(coreType)).thenReturn(appType);

        // Simulate core triggering the callback
        capturedCallback([(notificationId, coreType)]);

        // Assert
        expect(capturedId, notificationId);
        expect(capturedType, appType);
        verify(mockNotificationTypeMapper.map(coreType)).called(1);
      });
    });
  });
}
