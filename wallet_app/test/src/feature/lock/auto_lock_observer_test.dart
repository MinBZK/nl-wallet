import 'dart:async';
import 'dart:ui';

import 'package:clock/clock.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/service/navigation_service.dart';
import 'package:wallet/src/data/service/semantics_event_service.dart';
import 'package:wallet/src/domain/model/configuration/flutter_app_configuration.dart';
import 'package:wallet/src/domain/usecase/wallet/is_wallet_registered_and_unlocked_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/lock_wallet_usecase.dart';
import 'package:wallet/src/feature/lock/auto_lock_observer.dart';
import 'package:wallet/src/feature/lock/widget/interaction_detector.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.mocks.dart';

void main() {
  const idleLockTimeout = Duration(seconds: 10);
  const idleWarningTimeout = Duration(seconds: 5);
  const backgroundLockTimeout = Duration(seconds: 1);

  late MockAutoLockService mockAutoLockService;
  late FlutterAppConfiguration mockConfiguration;
  late MockIsWalletRegisteredAndUnlockedUseCase mockIsWalletRegisteredAndUnlockedUseCase;
  late MockLockWalletUseCase mockLockWalletUseCase;
  late MockSemanticsEventService mockSemanticsEventService;
  late MockNavigationService mockNavigationService;
  late StreamController<void> activityStreamController;
  late StreamController<SemanticsActionEvent> semanticsStreamController;

  setUp(() {
    mockAutoLockService = MockAutoLockService();
    mockConfiguration = const FlutterAppConfiguration(
      idleLockTimeout: idleLockTimeout,
      idleWarningTimeout: idleWarningTimeout,
      backgroundLockTimeout: backgroundLockTimeout,
      staticAssetsBaseUrl: 'https://example.org',
      version: 1,
    );
    mockIsWalletRegisteredAndUnlockedUseCase = MockIsWalletRegisteredAndUnlockedUseCase();
    mockLockWalletUseCase = MockLockWalletUseCase();
    mockSemanticsEventService = MockSemanticsEventService();
    mockNavigationService = MockNavigationService();
    activityStreamController = StreamController<void>.broadcast();
    semanticsStreamController = StreamController<SemanticsActionEvent>.broadcast();

    when(mockAutoLockService.activityStream).thenAnswer((_) => activityStreamController.stream);
    when(mockSemanticsEventService.actionEventStream).thenAnswer((_) => semanticsStreamController.stream);
    when(mockAutoLockService.autoLockEnabled).thenReturn(true);
  });

  tearDown(() {
    activityStreamController.close();
    semanticsStreamController.close();
  });

  Future<void> pumpAutoLockObserver(
    WidgetTester tester, {
    AppLifecycleState initialState = AppLifecycleState.resumed,
  }) async {
    tester.binding.handleAppLifecycleStateChanged(initialState);
    await tester.pumpWidgetWithAppWrapper(
      AutoLockObserver(
        autoLockService: mockAutoLockService,
        configuration: mockConfiguration,
        child: const SizedBox(),
      ),
      providers: [
        RepositoryProvider<IsWalletRegisteredAndUnlockedUseCase>.value(value: mockIsWalletRegisteredAndUnlockedUseCase),
        RepositoryProvider<LockWalletUseCase>.value(value: mockLockWalletUseCase),
        RepositoryProvider<SemanticsEventService>.value(value: mockSemanticsEventService),
        RepositoryProvider<NavigationService>.value(value: mockNavigationService),
      ],
    );
  }

  testWidgets('interaction resets idle timeout', (WidgetTester tester) async {
    await pumpAutoLockObserver(tester, initialState: AppLifecycleState.paused);

    await tester.tap(find.byType(InteractionDetector), warnIfMissed: false);

    verify(mockAutoLockService.resetIdleTimeout()).called(1);
  });

  testWidgets('app going to background and coming back locks wallet if timeout exceeded', (WidgetTester tester) async {
    DateTime time = DateTime.now();
    await withClock(Clock(() => time), () async {
      await pumpAutoLockObserver(tester);
      final binding = tester.binding;

      binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
      await tester.pump();

      // Time passes (more than timeout limit)
      time = time.add(backgroundLockTimeout + const Duration(milliseconds: 100));

      binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pump();

      verify(mockLockWalletUseCase.invoke()).called(1);
    });
  });

  testWidgets('app going to background and coming back does NOT lock wallet if timeout not exceeded', (
    WidgetTester tester,
  ) async {
    DateTime time = DateTime.now();
    await withClock(Clock(() => time), () async {
      await pumpAutoLockObserver(tester);
      final binding = tester.binding;

      binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
      await tester.pump();

      // Time passes (less than timeout limit)
      time = time.add(backgroundLockTimeout - const Duration(milliseconds: 100));

      binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pump();

      verifyNever(mockLockWalletUseCase.invoke());
    });
  });

  testWidgets('locks wallet after idle timeout', (WidgetTester tester) async {
    when(mockIsWalletRegisteredAndUnlockedUseCase.invoke()).thenAnswer((_) async => true);
    await pumpAutoLockObserver(tester);

    activityStreamController.add(null);

    // pump duration idleLockTimeout and then some
    await tester.pump(idleLockTimeout + const Duration(milliseconds: 100));

    verify(mockLockWalletUseCase.invoke()).called(1);
  });
}
