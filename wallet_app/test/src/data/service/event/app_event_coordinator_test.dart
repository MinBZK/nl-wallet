import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/service/event/app_event_coordinator.dart';

import '../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockWalletRepository mockWalletRepository;
  late MockAppEventListener mockListener1;
  late MockAppEventListener mockListener2;
  late AppEventCoordinator coordinator;
  late StreamController<bool> isLockedStreamController;

  setUp(() {
    mockWalletRepository = MockWalletRepository();
    mockListener1 = MockAppEventListener();
    mockListener2 = MockAppEventListener();

    isLockedStreamController = StreamController<bool>();
    when(mockWalletRepository.isLockedStream).thenAnswer((_) => isLockedStreamController.stream);

    coordinator = AppEventCoordinator(mockWalletRepository, [mockListener1]);
  });

  tearDown(() {
    coordinator.dispose();
    isLockedStreamController.close();
  });

  test('should broadcast onDashboardShown to all listeners', () {
    coordinator.addListener(mockListener2);
    coordinator.onDashboardShown();
    verify(mockListener1.onDashboardShown()).called(1);
    verify(mockListener2.onDashboardShown()).called(1);
  });

  test('should broadcast onWalletUnlocked to all listeners', () {
    coordinator.addListener(mockListener2);
    coordinator.onWalletUnlocked();
    verify(mockListener1.onWalletUnlocked()).called(1);
    verify(mockListener2.onWalletUnlocked()).called(1);
  });

  test('should broadcast onWalletLocked to all listeners', () {
    coordinator.addListener(mockListener2);
    coordinator.onWalletLocked();
    verify(mockListener1.onWalletLocked()).called(1);
    verify(mockListener2.onWalletLocked()).called(1);
  });

  test('should add a listener', () {
    coordinator.addListener(mockListener2);
    coordinator.onDashboardShown();
    verify(mockListener1.onDashboardShown()).called(1);
    verify(mockListener2.onDashboardShown()).called(1);
  });

  test('should remove a listener', () {
    coordinator.removeListener(mockListener1);
    coordinator.onDashboardShown();
    verifyNever(mockListener1.onDashboardShown());
  });

  group('lock state changes', () {
    test('should not trigger any event when app starts (locked)', () async {
      isLockedStreamController.add(true);
      await Future.delayed(Duration.zero);
      verifyNever(mockListener1.onWalletLocked());
      verifyNever(mockListener1.onWalletUnlocked());
    });

    test('should trigger onWalletUnlocked when wallet is unlocked', () async {
      isLockedStreamController.add(true); // initial locked state, should be skipped
      isLockedStreamController.add(false); // unlocked
      await Future.delayed(Duration.zero);
      verify(mockListener1.onWalletUnlocked()).called(1);
      verifyNever(mockListener1.onWalletLocked());
    });

    test('should trigger onWalletLocked when wallet is locked after being unlocked', () async {
      isLockedStreamController.add(true); // initial locked state, skipped
      isLockedStreamController.add(false); // unlocked
      await Future.delayed(Duration.zero);
      isLockedStreamController.add(true); // locked
      await Future.delayed(Duration.zero);
      verify(mockListener1.onWalletUnlocked()).called(1);
      verify(mockListener1.onWalletLocked()).called(1);
    });
  });

  test('dispose should cancel stream subscription', () async {
    coordinator.dispose();

    isLockedStreamController.add(false);
    await Future.delayed(Duration.zero);

    verifyNever(mockListener1.onWalletUnlocked());
  });
}
