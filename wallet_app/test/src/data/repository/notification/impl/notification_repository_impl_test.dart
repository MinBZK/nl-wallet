import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/notification/impl/notification_repository_impl.dart';
import 'package:wallet/src/data/repository/notification/notification_repository.dart';
import 'package:wallet/src/domain/model/card/format/attestation_format.dart';
import 'package:wallet/src/domain/model/configuration/flutter_app_configuration.dart';
import 'package:wallet/src/domain/model/notification/app_notification.dart';
import 'package:wallet/src/domain/model/pid/pid_attestation.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../../mocks/core_mock_data.dart';
import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockTypedWalletCore mockCore;
  late MockMapper<core.AppNotification, AppNotification> mockNotificationMapper;
  late MockMapper<core.NotificationType, NotificationType> mockNotificationTypeMapper;
  late MockMapper<core.FlutterConfiguration, FlutterAppConfiguration> mockConfigMapper;
  late MockNotificationSettingsStore mockNotificationSettingsStore;
  late NotificationRepository notificationRepository;

  setUp(() {
    mockCore = MockTypedWalletCore();
    mockNotificationMapper = MockMapper<core.AppNotification, AppNotification>();
    mockNotificationTypeMapper = MockMapper<core.NotificationType, NotificationType>();
    mockConfigMapper = MockMapper<core.FlutterConfiguration, FlutterAppConfiguration>();
    mockNotificationSettingsStore = MockNotificationSettingsStore();
    notificationRepository = NotificationRepositoryImpl(
      mockCore,
      mockNotificationMapper,
      mockNotificationTypeMapper,
      mockNotificationSettingsStore,
      mockConfigMapper,
    );

    when(mockCore.observeConfig()).thenAnswer((_) => Stream.value(CoreMockData.flutterConfiguration));
    when(mockConfigMapper.map(any)).thenReturn(
      const FlutterAppConfiguration(
        idleLockTimeout: Duration.zero,
        idleWarningTimeout: Duration.zero,
        backgroundLockTimeout: Duration.zero,
        staticAssetsBaseUrl: '',
        pidAttestations: [],
        maintenanceWindow: null,
        version: '',
        environment: '',
      ),
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
        verify(mockCore.observeConfig()).called(1);
        verify(mockNotificationMapper.mapList(coreNotifications)).called(1);
        verifyNoMoreInteractions(mockCore);
        verifyNoMoreInteractions(mockNotificationMapper);
      });

      test('should NOT filter PID notifications when filterDuplicatePidNotifications is false', () async {
        final pidSdJwt = WalletMockData.card.copyWith(attestationType: 'pid', format: AttestationFormat.sdJwt);
        final pidMdoc = WalletMockData.card.copyWith(
          attestationId: 'id2',
          attestationType: 'pid',
          format: AttestationFormat.mdoc,
        );

        final notificationSdJwt = AppNotification(
          id: 1,
          type: NotificationType.cardExpired(card: pidSdJwt),
          displayTargets: [const Dashboard()],
        );
        final notificationMdoc = AppNotification(
          id: 2,
          type: NotificationType.cardExpired(card: pidMdoc),
          displayTargets: [const Dashboard()],
        );

        final coreNotifications = [
          const core.AppNotification(
            id: 1,
            targets: [],
            typ: core.NotificationType_CardExpired(card: CoreMockData.attestation),
          ),
          const core.AppNotification(
            id: 2,
            targets: [],
            typ: core.NotificationType_CardExpired(card: CoreMockData.attestation),
          ),
        ];
        final appNotifications = [notificationSdJwt, notificationMdoc];

        when(mockCore.observeNotifications()).thenAnswer((_) => Stream.value(coreNotifications));
        when(mockNotificationMapper.mapList(coreNotifications)).thenReturn(appNotifications);

        await expectLater(
          notificationRepository.observeNotifications(filterDuplicatePidNotifications: false),
          emits(appNotifications),
        );

        verifyNever(mockCore.observeConfig()); // Filtering relies on config, make sure it's not fetched.
      });

      test('should filter duplicate PID notifications based on priority', () async {
        final pidSdJwt = WalletMockData.card.copyWith(attestationType: 'pid', format: AttestationFormat.sdJwt);
        final pidMdoc = WalletMockData.card.copyWith(
          attestationId: 'id2',
          attestationType: 'pid',
          format: AttestationFormat.mdoc,
        );

        final notificationSdJwt = AppNotification(
          id: 1,
          type: NotificationType.cardExpired(card: pidSdJwt),
          displayTargets: [const Dashboard()],
        );
        final notificationMdoc = AppNotification(
          id: 2,
          type: NotificationType.cardExpired(card: pidMdoc),
          displayTargets: [const Dashboard()],
        );

        final coreNotifications = [
          const core.AppNotification(
            id: 1,
            targets: [],
            typ: core.NotificationType_CardExpired(card: CoreMockData.attestation),
          ),
          const core.AppNotification(
            id: 2,
            targets: [],
            typ: core.NotificationType_CardExpired(card: CoreMockData.attestation),
          ),
        ];
        final mappedNotifications = [notificationSdJwt, notificationMdoc];

        when(mockCore.observeNotifications()).thenAnswer((_) => Stream.value(coreNotifications));
        when(mockNotificationMapper.mapList(coreNotifications)).thenReturn(mappedNotifications);

        // SD-JWT has higher priority
        when(mockConfigMapper.map(any)).thenReturn(
          const FlutterAppConfiguration(
            idleLockTimeout: Duration.zero,
            idleWarningTimeout: Duration.zero,
            backgroundLockTimeout: Duration.zero,
            staticAssetsBaseUrl: '',
            pidAttestations: [
              PidAttestation(attestationType: 'pid', format: AttestationFormat.sdJwt),
              PidAttestation(attestationType: 'pid', format: AttestationFormat.mdoc),
            ],
            maintenanceWindow: null,
            version: '',
            environment: '',
          ),
        );

        await expectLater(notificationRepository.observeNotifications(), emits([notificationSdJwt]));
      });

      test('should NOT filter non-PID notifications even if they look similar', () async {
        final nonPidCard1 = WalletMockData.card.copyWith(attestationId: 'id1', attestationType: 'other');
        final pidCard1 = WalletMockData.card.copyWith(attestationId: 'pid1', attestationType: 'pid');

        final notification1 = AppNotification(
          id: 1,
          type: NotificationType.cardExpired(card: nonPidCard1),
          displayTargets: [const Dashboard()],
        );
        final notification2 = AppNotification(
          id: 2,
          type: NotificationType.cardExpired(card: pidCard1),
          displayTargets: [const Dashboard()],
        );

        final coreNotifications = [
          const core.AppNotification(
            id: 1,
            targets: [],
            typ: core.NotificationType_CardExpired(card: CoreMockData.attestation),
          ),
          const core.AppNotification(
            id: 2,
            targets: [],
            typ: core.NotificationType_CardExpired(card: CoreMockData.attestation),
          ),
        ];
        final mappedNotifications = [notification1, notification2];

        when(mockCore.observeNotifications()).thenAnswer((_) => Stream.value(coreNotifications));
        when(mockNotificationMapper.mapList(coreNotifications)).thenReturn(mappedNotifications);

        // Config has PID but it's different
        when(mockConfigMapper.map(any)).thenReturn(
          const FlutterAppConfiguration(
            idleLockTimeout: Duration.zero,
            idleWarningTimeout: Duration.zero,
            backgroundLockTimeout: Duration.zero,
            staticAssetsBaseUrl: '',
            pidAttestations: [
              PidAttestation(attestationType: 'pid', format: AttestationFormat.sdJwt),
            ],
            maintenanceWindow: null,
            version: '',
            environment: '',
          ),
        );

        await expectLater(notificationRepository.observeNotifications(), emits(mappedNotifications));
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
      test('should call core.setupNotificationCallback and map the results when triggered', () async {
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
        final capturedCallback = captured.first as dynamic;

        // Mock the mapper's behavior for when the callback is triggered
        when(mockNotificationTypeMapper.map(coreType)).thenReturn(appType);

        // Simulate core triggering the callback
        await capturedCallback([(notificationId, coreType)]);

        // Assert
        expect(capturedId, notificationId);
        expect(capturedType, appType);
        verify(mockNotificationTypeMapper.map(coreType)).called(1);
      });

      test('should NOT filter PID notifications in direct callback when flag is false', () async {
        // Arrange
        final pidSdJwt = WalletMockData.card.copyWith(attestationType: 'pid', format: AttestationFormat.sdJwt);
        final pidMdoc = WalletMockData.card.copyWith(
          attestationId: 'id2',
          attestationType: 'pid',
          format: AttestationFormat.mdoc,
        );

        final typeSdJwt = NotificationType.cardExpired(card: pidSdJwt);
        final typeMdoc = NotificationType.cardExpired(card: pidMdoc);

        const coreTypeSdJwt = core.NotificationType_CardExpired(card: CoreMockData.attestation);
        const coreTypeMdoc = core.NotificationType_CardExpired(card: CoreMockData.altAttestation);

        when(mockNotificationTypeMapper.map(coreTypeSdJwt)).thenReturn(typeSdJwt);
        when(mockNotificationTypeMapper.map(coreTypeMdoc)).thenReturn(typeMdoc);

        final List<(int, NotificationType)> capturedNotifications = [];

        // Act
        notificationRepository.setDirectNotificationCallback(
          (id, type) => capturedNotifications.add((id, type)),
          filterDuplicatePidNotifications: false,
        );

        final captured = verify(mockCore.setupNotificationCallback(captureAny)).captured;
        final capturedCallback = captured.first as dynamic;

        // Simulate core triggering callback with two related PID notifications
        await capturedCallback([(1, coreTypeSdJwt), (2, coreTypeMdoc)]);

        // Assert
        expect(capturedNotifications, hasLength(2));
        verifyNever(mockCore.observeConfig());
      });

      test('should filter duplicate PID notifications in direct callback based on priority', () async {
        // Arrange
        final pidSdJwt = WalletMockData.card.copyWith(attestationType: 'pid', format: AttestationFormat.sdJwt);
        final pidMdoc = WalletMockData.card.copyWith(
          attestationId: 'id2',
          attestationType: 'pid',
          format: AttestationFormat.mdoc,
        );

        final typeSdJwt = NotificationType.cardExpired(card: pidSdJwt);
        final typeMdoc = NotificationType.cardExpired(card: pidMdoc);

        const coreTypeSdJwt = core.NotificationType_CardExpired(card: CoreMockData.attestation);
        const coreTypeMdoc = core.NotificationType_CardExpired(card: CoreMockData.altAttestation);

        // SD-JWT has higher priority
        when(mockConfigMapper.map(any)).thenReturn(
          const FlutterAppConfiguration(
            idleLockTimeout: Duration.zero,
            idleWarningTimeout: Duration.zero,
            backgroundLockTimeout: Duration.zero,
            staticAssetsBaseUrl: '',
            pidAttestations: [
              PidAttestation(attestationType: 'pid', format: AttestationFormat.sdJwt),
              PidAttestation(attestationType: 'pid', format: AttestationFormat.mdoc),
            ],
            maintenanceWindow: null,
            version: '',
            environment: '',
          ),
        );

        when(mockNotificationTypeMapper.map(coreTypeSdJwt)).thenReturn(typeSdJwt);
        when(mockNotificationTypeMapper.map(coreTypeMdoc)).thenReturn(typeMdoc);

        final List<(int, NotificationType)> capturedNotifications = [];

        // Act
        notificationRepository.setDirectNotificationCallback((id, type) {
          capturedNotifications.add((id, type));
        });

        final captured = verify(mockCore.setupNotificationCallback(captureAny)).captured;
        final capturedCallback = captured.first as dynamic;

        // Simulate core triggering callback with two related PID notifications
        await capturedCallback([(1, coreTypeSdJwt), (2, coreTypeMdoc)]);

        // Assert
        expect(capturedNotifications, hasLength(1));
        expect(capturedNotifications.first.$1, 1);
        expect(capturedNotifications.first.$2, typeSdJwt);
      });

      test('should NOT filter non-PID notifications in direct callback', () async {
        // Arrange
        final nonPidCard1 = WalletMockData.card.copyWith(attestationId: 'id1', attestationType: 'other');
        final nonPidCard2 = WalletMockData.card.copyWith(attestationId: 'id2', attestationType: 'other');

        final type1 = NotificationType.cardExpired(card: nonPidCard1);
        final type2 = NotificationType.cardExpired(card: nonPidCard2);

        const coreType1 = core.NotificationType_CardExpired(card: CoreMockData.attestation);
        const coreType2 = core.NotificationType_CardExpired(card: CoreMockData.altAttestation);

        when(mockConfigMapper.map(any)).thenReturn(
          const FlutterAppConfiguration(
            idleLockTimeout: Duration.zero,
            idleWarningTimeout: Duration.zero,
            backgroundLockTimeout: Duration.zero,
            staticAssetsBaseUrl: '',
            pidAttestations: [
              PidAttestation(attestationType: 'pid', format: AttestationFormat.sdJwt),
            ],
            maintenanceWindow: null,
            version: '',
            environment: '',
          ),
        );

        when(mockNotificationTypeMapper.map(coreType1)).thenReturn(type1);
        when(mockNotificationTypeMapper.map(coreType2)).thenReturn(type2);

        final List<(int, NotificationType)> capturedNotifications = [];

        // Act
        notificationRepository.setDirectNotificationCallback((id, type) {
          capturedNotifications.add((id, type));
        });

        final captured = verify(mockCore.setupNotificationCallback(captureAny)).captured;
        final capturedCallback = captured.first as dynamic;

        // Simulate core triggering callback with two related non-PID notifications
        await capturedCallback([(1, coreType1), (2, coreType2)]);

        // Assert
        expect(capturedNotifications, hasLength(2));
        expect(capturedNotifications[0].$1, 1);
        expect(capturedNotifications[0].$2, type1);
        expect(capturedNotifications[1].$1, 2);
        expect(capturedNotifications[1].$2, type2);
      });
    });
  });
}
