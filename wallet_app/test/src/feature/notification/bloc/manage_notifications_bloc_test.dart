import 'dart:async';

import 'package:app_settings/app_settings_method_channel.dart';
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:wallet/src/data/service/app_lifecycle_service.dart';
import 'package:wallet/src/domain/model/permission/permission_check_result.dart';
import 'package:wallet/src/feature/notification/bloc/manage_notifications_bloc.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  late MockCheckPermissionUseCase checkPermissionUseCase;
  late MockRequestPermissionUseCase requestPermissionUseCase;
  late MockObservePushNotificationsSettingUseCase observePushNotificationsSettingUseCase;
  late MockSetPushNotificationsSettingUseCase setPushNotificationsSettingUseCase;
  late AppLifecycleService lifecycleService;

  final List<String> methodCallLog = <String>[];

  setUp(() {
    checkPermissionUseCase = MockCheckPermissionUseCase();
    requestPermissionUseCase = MockRequestPermissionUseCase();
    observePushNotificationsSettingUseCase = MockObservePushNotificationsSettingUseCase();
    setPushNotificationsSettingUseCase = MockSetPushNotificationsSettingUseCase();
    lifecycleService = AppLifecycleService();

    when(setPushNotificationsSettingUseCase.invoke(enabled: anyNamed('enabled'))).thenAnswer((_) async {});

    methodCallLog.clear();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
      MethodChannelAppSettings().methodChannel,
      (methodCall) async {
        methodCallLog.add(methodCall.method);
        return null;
      },
    );
  });

  blocTest(
    'verify initial state',
    build: () => ManageNotificationsBloc(
      checkPermissionUseCase,
      requestPermissionUseCase,
      observePushNotificationsSettingUseCase,
      setPushNotificationsSettingUseCase,
      lifecycleService,
    ),
    setUp: () {
      when(observePushNotificationsSettingUseCase.invoke()).thenAnswer((_) => Stream.value(true));
    },
    verify: (bloc) => expect(bloc.state, const ManageNotificationsInitial()),
  );

  group('on load', () {
    blocTest(
      'verify loaded state when enabled and permission granted',
      build: () => ManageNotificationsBloc(
        checkPermissionUseCase,
        requestPermissionUseCase,
        observePushNotificationsSettingUseCase,
        setPushNotificationsSettingUseCase,
        lifecycleService,
      ),
      act: (bloc) => bloc.add(const ManageNotificationsLoadTriggered()),
      setUp: () {
        when(observePushNotificationsSettingUseCase.invoke()).thenAnswer((_) => Stream.value(true));
        when(
          checkPermissionUseCase.invoke(Permission.notification),
        ).thenAnswer((_) async => const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
      },
      expect: () => [const ManageNotificationsLoaded(pushEnabled: true)],
    );

    blocTest(
      'verify loaded state when enabled and permission denied',
      build: () => ManageNotificationsBloc(
        checkPermissionUseCase,
        requestPermissionUseCase,
        observePushNotificationsSettingUseCase,
        setPushNotificationsSettingUseCase,
        lifecycleService,
      ),
      act: (bloc) => bloc.add(const ManageNotificationsLoadTriggered()),
      setUp: () {
        when(observePushNotificationsSettingUseCase.invoke()).thenAnswer((_) => Stream.value(true));
        when(
          checkPermissionUseCase.invoke(Permission.notification),
        ).thenAnswer((_) async => const PermissionCheckResult(isGranted: false, isPermanentlyDenied: false));
      },
      expect: () => [const ManageNotificationsLoaded(pushEnabled: false)],
    );

    blocTest(
      'verify loaded state when disabled and permission granted',
      build: () => ManageNotificationsBloc(
        checkPermissionUseCase,
        requestPermissionUseCase,
        observePushNotificationsSettingUseCase,
        setPushNotificationsSettingUseCase,
        lifecycleService,
      ),
      act: (bloc) => bloc.add(const ManageNotificationsLoadTriggered()),
      setUp: () {
        when(observePushNotificationsSettingUseCase.invoke()).thenAnswer((_) => Stream.value(false));
        when(
          checkPermissionUseCase.invoke(Permission.notification),
        ).thenAnswer((_) async => const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
      },
      expect: () => [const ManageNotificationsLoaded(pushEnabled: false)],
    );
  });

  group('on toggle', () {
    blocTest(
      'verify toggle off',
      build: () => ManageNotificationsBloc(
        checkPermissionUseCase,
        requestPermissionUseCase,
        observePushNotificationsSettingUseCase,
        setPushNotificationsSettingUseCase,
        lifecycleService,
      ),
      seed: () => const ManageNotificationsLoaded(pushEnabled: true),
      act: (bloc) => bloc.add(const ManageNotificationsPushNotificationsToggled()),
      expect: () => [const ManageNotificationsLoaded(pushEnabled: false)],
      verify: (_) {
        verify(setPushNotificationsSettingUseCase.invoke(enabled: false));
      },
    );

    blocTest(
      'verify toggle on (permission already granted)',
      build: () => ManageNotificationsBloc(
        checkPermissionUseCase,
        requestPermissionUseCase,
        observePushNotificationsSettingUseCase,
        setPushNotificationsSettingUseCase,
        lifecycleService,
      ),
      seed: () => const ManageNotificationsLoaded(pushEnabled: false),
      setUp: () {
        when(
          checkPermissionUseCase.invoke(Permission.notification),
        ).thenAnswer((_) async => const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
      },
      act: (bloc) => bloc.add(const ManageNotificationsPushNotificationsToggled()),
      expect: () => [const ManageNotificationsLoaded(pushEnabled: true)],
      verify: (_) {
        verify(setPushNotificationsSettingUseCase.invoke(enabled: true));
      },
    );

    blocTest(
      'verify toggle on (permission permanently denied) -> trigger os level settings',
      build: () => ManageNotificationsBloc(
        checkPermissionUseCase,
        requestPermissionUseCase,
        observePushNotificationsSettingUseCase,
        setPushNotificationsSettingUseCase,
        lifecycleService,
      ),
      seed: () => const ManageNotificationsLoaded(pushEnabled: false),
      setUp: () {
        when(
          checkPermissionUseCase.invoke(Permission.notification),
        ).thenAnswer((_) async => const PermissionCheckResult(isGranted: false, isPermanentlyDenied: true));
      },
      act: (bloc) => bloc.add(const ManageNotificationsPushNotificationsToggled()),
      expect: () => [],
      verify: (_) {
        expect(methodCallLog, <String>['openSettings']);
      },
    );

    blocTest(
      'verify toggle on (request and grant permission)',
      build: () => ManageNotificationsBloc(
        checkPermissionUseCase,
        requestPermissionUseCase,
        observePushNotificationsSettingUseCase,
        setPushNotificationsSettingUseCase,
        lifecycleService,
      ),
      seed: () => const ManageNotificationsLoaded(pushEnabled: false),
      setUp: () {
        when(
          checkPermissionUseCase.invoke(Permission.notification),
        ).thenAnswer((_) async => const PermissionCheckResult(isGranted: false, isPermanentlyDenied: false));
        when(
          requestPermissionUseCase.invoke(Permission.notification),
        ).thenAnswer((_) async => const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
      },
      act: (bloc) => bloc.add(const ManageNotificationsPushNotificationsToggled()),
      expect: () => [const ManageNotificationsLoaded(pushEnabled: true)],
      verify: (_) {
        verify(requestPermissionUseCase.invoke(Permission.notification));
        verify(setPushNotificationsSettingUseCase.invoke(enabled: true));
      },
    );

    blocTest(
      'verify toggle on (request and deny permission)',
      build: () => ManageNotificationsBloc(
        checkPermissionUseCase,
        requestPermissionUseCase,
        observePushNotificationsSettingUseCase,
        setPushNotificationsSettingUseCase,
        lifecycleService,
      ),
      seed: () => const ManageNotificationsLoaded(pushEnabled: false),
      setUp: () {
        when(
          checkPermissionUseCase.invoke(Permission.notification),
        ).thenAnswer((_) async => const PermissionCheckResult(isGranted: false, isPermanentlyDenied: false));
        when(
          requestPermissionUseCase.invoke(Permission.notification),
        ).thenAnswer((_) async => const PermissionCheckResult(isGranted: false, isPermanentlyDenied: false));
      },
      act: (bloc) => bloc.add(const ManageNotificationsPushNotificationsToggled()),
      expect: () => [/* no state changer */],
      verify: (_) {
        verify(requestPermissionUseCase.invoke(Permission.notification));
        verify(setPushNotificationsSettingUseCase.invoke(enabled: false));
      },
    );

    blocTest(
      'verify onResume triggers permission and settings check and emits result',
      build: () => ManageNotificationsBloc(
        checkPermissionUseCase,
        requestPermissionUseCase,
        observePushNotificationsSettingUseCase,
        setPushNotificationsSettingUseCase,
        lifecycleService,
      ),
      seed: () => const ManageNotificationsLoaded(pushEnabled: false),
      setUp: () {
        when(observePushNotificationsSettingUseCase.invoke()).thenAnswer((_) => Stream.value(true));
        when(
          checkPermissionUseCase.invoke(Permission.notification),
        ).thenAnswer((_) async => const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
        when(
          requestPermissionUseCase.invoke(Permission.notification),
        ).thenAnswer((_) async => const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
      },
      act: (bloc) => lifecycleService.notifyStateChanged(.resumed),
      expect: () => [const ManageNotificationsLoaded(pushEnabled: true)],
      verify: (_) {
        verify(observePushNotificationsSettingUseCase.invoke());
        verify(checkPermissionUseCase.invoke(Permission.notification));
      },
    );
  });
}
