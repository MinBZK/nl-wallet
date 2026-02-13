import 'dart:ui';

import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/data/service/local_notification_service.dart';
import 'package:wallet/src/domain/model/notification/notification_channel.dart';
import 'package:wallet/src/domain/model/notification/os_notification.dart';

import '../../mocks/wallet_mocks.mocks.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  late MockObserveOsNotificationsUseCase mockObserveOsNotificationsUseCase;
  late MockSetDirectOsNotificationCallbackUsecase mockSetDirectOsNotificationCallbackUsecase;
  late MockActiveLocaleProvider mockActiveLocaleProvider;
  late MockNavigationService mockNavigationService;
  late MockFlutterLocalNotificationsPlugin mockPlugin;

  late BehaviorSubject<List<OsNotification>> notificationStream;

  setUp(() {
    mockObserveOsNotificationsUseCase = MockObserveOsNotificationsUseCase();
    mockSetDirectOsNotificationCallbackUsecase = MockSetDirectOsNotificationCallbackUsecase();
    mockActiveLocaleProvider = MockActiveLocaleProvider();
    mockNavigationService = MockNavigationService();
    mockPlugin = MockFlutterLocalNotificationsPlugin();

    notificationStream = BehaviorSubject<List<OsNotification>>();
    when(mockObserveOsNotificationsUseCase.invoke()).thenAnswer((_) => notificationStream.stream);

    // Setup default responses for plugin mock
    when(
      mockPlugin.initialize(
        any,
        onDidReceiveNotificationResponse: anyNamed('onDidReceiveNotificationResponse'),
        onDidReceiveBackgroundNotificationResponse: anyNamed('onDidReceiveNotificationResponse'),
      ),
    ).thenAnswer((_) async => true);
    when(mockPlugin.cancelAllPendingNotifications()).thenAnswer((_) async => {});
    when(
      mockPlugin.zonedSchedule(any, any, any, any, any, androidScheduleMode: anyNamed('androidScheduleMode')),
    ).thenAnswer((_) async => {});
    when(
      mockPlugin.show(any, any, any, any, payload: anyNamed('payload')),
    ).thenAnswer((_) async => {});
  });

  tearDown(() {
    notificationStream.close();
  });

  group('LocalNotificationService', () {
    test('initializes and listens to notifications', () async {
      LocalNotificationService(
        mockObserveOsNotificationsUseCase,
        mockSetDirectOsNotificationCallbackUsecase,
        mockActiveLocaleProvider,
        mockNavigationService,
        factory: () => mockPlugin,
      );

      // Verify initialization
      verify(
        mockPlugin.initialize(
          any,
          onDidReceiveNotificationResponse: anyNamed('onDidReceiveNotificationResponse'),
          onDidReceiveBackgroundNotificationResponse: anyNamed('onDidReceiveBackgroundNotificationResponse'),
        ),
      ).called(1);

      // Wait for initialization to complete internally
      await Future.microtask(() {});

      // Verify use cases invoked
      verify(mockObserveOsNotificationsUseCase.invoke()).called(1);
      verify(mockSetDirectOsNotificationCallbackUsecase.invoke(any)).called(1);
    });

    test('schedules notifications when stream emits', () async {
      LocalNotificationService(
        mockObserveOsNotificationsUseCase,
        mockSetDirectOsNotificationCallbackUsecase,
        mockActiveLocaleProvider,
        mockNavigationService,
        factory: () => mockPlugin,
      );

      // Wait for initialization to complete internally
      await Future.microtask(() {});

      final notification = OsNotification(
        id: 1,
        channel: NotificationChannel.cardUpdates,
        title: 'Title',
        body: 'Body',
        notifyAt: DateTime.now().add(const Duration(minutes: 1)),
      );

      notificationStream.add([notification]);

      // Wait for async processing in _onNotificationUpdate
      await Future.delayed(const Duration(milliseconds: 10));

      verify(mockPlugin.cancelAllPendingNotifications()).called(1);
      verify(
        mockPlugin.zonedSchedule(
          notification.id,
          notification.title,
          notification.body,
          any,
          any,
          androidScheduleMode: .inexact,
          payload: anyNamed('payload'),
        ),
      ).called(1);
    });

    test('shows direct notification when callback is triggered', () async {
      void Function(OsNotification)? capturedCallback;
      when(mockSetDirectOsNotificationCallbackUsecase.invoke(any)).thenAnswer((invocation) {
        capturedCallback = invocation.positionalArguments.first as void Function(OsNotification);
      });

      LocalNotificationService(
        mockObserveOsNotificationsUseCase,
        mockSetDirectOsNotificationCallbackUsecase,
        mockActiveLocaleProvider,
        mockNavigationService,
        factory: () => mockPlugin,
      );

      // Allow async _initPlugin call in constructor to settle
      await Future.delayed(Duration.zero);

      expect(capturedCallback, isNotNull);

      final notification = OsNotification(
        id: 2,
        channel: NotificationChannel.cardUpdates,
        title: 'Direct Title',
        body: 'Direct Body',
        notifyAt: DateTime.now(),
      );

      capturedCallback!(notification);

      verify(
        mockPlugin.show(
          notification.id,
          notification.title,
          notification.body,
          any,
          payload: anyNamed('payload'),
        ),
      ).called(1);
    });

    group('Android details resolution', () {
      test('resolves correct channel name and description', () async {
        when(mockActiveLocaleProvider.activeLocale).thenReturn(const Locale('en'));

        void Function(OsNotification)? capturedCallback;
        when(mockSetDirectOsNotificationCallbackUsecase.invoke(any)).thenAnswer((invocation) {
          capturedCallback = invocation.positionalArguments.first as void Function(OsNotification);
        });

        LocalNotificationService(
          mockObserveOsNotificationsUseCase,
          mockSetDirectOsNotificationCallbackUsecase,
          mockActiveLocaleProvider,
          mockNavigationService,
          factory: () => mockPlugin,
        );

        // Allow async _initPlugin call in constructor to settle
        await Future.delayed(Duration.zero);

        final notification = OsNotification(
          id: 3,
          channel: NotificationChannel.cardUpdates,
          title: 'Title',
          body: 'Body',
          notifyAt: DateTime.now(),
        );

        capturedCallback!(notification);

        final VerificationResult verification = verify(
          mockPlugin.show(
            any,
            any,
            any,
            captureAny,
            payload: anyNamed('payload'),
          ),
        );

        final NotificationDetails details = verification.captured.single;
        expect(details.android?.channelId, NotificationChannel.cardUpdates.name);
        expect(details.android?.channelName, contains('Card status'));
        expect(details.android?.channelDescription, contains('Notifications about the status of your cards'));
      });
    });
  });
}
