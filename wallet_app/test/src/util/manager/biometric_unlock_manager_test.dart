import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/data/service/app_lifecycle_service.dart';
import 'package:wallet/src/util/manager/biometric_unlock_manager.dart';

import '../../mocks/wallet_mocks.dart';

void main() {
  late BiometricUnlockManager manager;
  late AppLifecycleService lifecycleService;
  late MockWalletRepository walletRepository;
  late BehaviorSubject<bool> mockWalletLockSubject;
  late MockIsBiometricLoginEnabledUseCase isBiometricLoginEnabledUseCase;

  // Test helper methods to manage the (mock) wallet lock state
  Future<void> unlockMockWallet() {
    mockWalletLockSubject.add(false);
    return Future.microtask(() {
      /* delay so stream gets processed */
    });
  }

  Future<void> lockMockWallet() {
    mockWalletLockSubject.add(true);
    return Future.microtask(() {
      /* delay so stream gets processed */
    });
  }

  Future<void> updateMockLifecycle(AppLifecycleState state) {
    lifecycleService.notifyStateChanged(state);
    return Future.delayed(const Duration(milliseconds: 150));
  }

  setUp(() async {
    lifecycleService = AppLifecycleService();

    // Initiate [WalletRepository] with locked wallet
    walletRepository = MockWalletRepository();
    mockWalletLockSubject = BehaviorSubject.seeded(true);
    when(walletRepository.isLockedStream).thenAnswer((_) => mockWalletLockSubject);

    // Default to biometrics being enabled, this class is pointless without biometrics
    isBiometricLoginEnabledUseCase = MockIsBiometricLoginEnabledUseCase();
    when(isBiometricLoginEnabledUseCase.invoke()).thenAnswer((_) async => true);

    manager = BiometricUnlockManager(
      lifecycleService,
      walletRepository,
      isBiometricLoginEnabledUseCase,
    );
  });

  test('shouldTriggerUnlock defaults to true so that it triggers on cold app start', () async {
    expect(manager.shouldTriggerUnlock, isTrue);
  });

  test('when unlocking the wallet, shouldTrigger unlock should be false', () async {
    await unlockMockWallet();
    expect(manager.shouldTriggerUnlock, isFalse);
  });

  test('locking the wallet should not change shouldTriggerUnlock', () async {
    await unlockMockWallet();
    expect(manager.shouldTriggerUnlock, isFalse);
    await lockMockWallet();
    expect(manager.shouldTriggerUnlock, isFalse);
  });

  test('backgrounding the app should result in shouldTriggerUnlock=true', () async {
    await unlockMockWallet();
    expect(manager.shouldTriggerUnlock, isFalse);
    await lockMockWallet();
    expect(manager.shouldTriggerUnlock, isFalse);
    await updateMockLifecycle(AppLifecycleState.hidden);
    expect(manager.shouldTriggerUnlock, isTrue);
  });

  test('backgrounding the app should not result in shouldTriggerUnlock=true when biometrics are disabled', () async {
    when(isBiometricLoginEnabledUseCase.invoke()).thenAnswer((_) async => false);
    await unlockMockWallet();
    expect(manager.shouldTriggerUnlock, isFalse);
    await lockMockWallet();
    await updateMockLifecycle(AppLifecycleState.hidden);
    expect(manager.shouldTriggerUnlock, isFalse);
  });

  test('backgrounding -> foregrounding the app while it is unlocked should not change shouldTriggerUnlock', () async {
    await unlockMockWallet();
    expect(manager.shouldTriggerUnlock, isFalse);
    await updateMockLifecycle(AppLifecycleState.hidden);
    await updateMockLifecycle(AppLifecycleState.resumed);
    expect(manager.shouldTriggerUnlock, isFalse);
  });

  test('backgrounding -> foregrounding the app when it is locked should change shouldTriggerUnlock', () async {
    await unlockMockWallet(); // Make sure shouldTriggerUnlock is set to false
    await lockMockWallet();
    expect(manager.shouldTriggerUnlock, isFalse);
    // Hide and resume the app while it's locked
    await updateMockLifecycle(AppLifecycleState.hidden);
    await updateMockLifecycle(AppLifecycleState.resumed);
    expect(manager.shouldTriggerUnlock, isTrue);
  });

  test('getAndSet behaves as expected', () async {
    // `Update to false while it's true
    expect(manager.getAndSetShouldTriggerUnlock(updatedValue: false), isTrue);
    // Update to true, while it's false
    expect(manager.getAndSetShouldTriggerUnlock(updatedValue: true), isFalse);
    // Update (keep) at true, while it's true
    expect(manager.getAndSetShouldTriggerUnlock(updatedValue: true), isTrue);
    // Verify it's still true
    expect(manager.shouldTriggerUnlock, isTrue);
  });

  test(
      'verify shouldTriggerUnlock is set correctly when the app is locked in resumed callback (e.g. by AutoLockObserver)',
      () async {
    await unlockMockWallet(); // Make sure shouldTriggerUnlock is set to false
    expect(manager.shouldTriggerUnlock, isFalse);

    // Put the app in the background
    await updateMockLifecycle(AppLifecycleState.hidden);
    expect(manager.shouldTriggerUnlock, isTrue);

    // Set lifecycle & lock WITHOUT helper method to simulate race conditions.
    await Future.wait([
      Future.microtask(() => lifecycleService.notifyStateChanged(AppLifecycleState.resumed)),
      Future.microtask(() => mockWalletLockSubject.add(true)),
    ]);

    expect(manager.shouldTriggerUnlock, isTrue);
  });
}
