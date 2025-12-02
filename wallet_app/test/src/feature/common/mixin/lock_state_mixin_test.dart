import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/domain/usecase/wallet/observe_wallet_locked_usecase.dart';
import 'package:wallet/src/feature/common/mixin/lock_state_mixin.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockObserveWalletLockedUseCase observeWalletLockedUseCase;
  late BehaviorSubject<bool> isLockedStream;

  setUp(() {
    isLockedStream = BehaviorSubject.seeded(false);
    observeWalletLockedUseCase = MockObserveWalletLockedUseCase();
    when(observeWalletLockedUseCase.invoke()).thenAnswer((_) => isLockedStream);
  });

  testWidgets(
    'verify no initial callbacks',
    (WidgetTester tester) async {
      int onLockCallCount = 0;
      int onUnlockCallCount = 0;
      await tester.pumpWidgetWithAppWrapper(
        _LockStateTester(
          onLock: () => onLockCallCount++,
          onUnlock: () => onUnlockCallCount++,
        ).withDependency<ObserveWalletLockedUseCase>((context) => observeWalletLockedUseCase),
      );
      expect(onLockCallCount, 0);
      expect(onUnlockCallCount, 0);
    },
  );

  testWidgets(
    'verify onLock callbacks',
    (WidgetTester tester) async {
      int onLockCallCount = 0;
      int onUnlockCallCount = 0;
      await tester.pumpWidgetWithAppWrapper(
        _LockStateTester(
          onLock: () => onLockCallCount++,
          onUnlock: () => onUnlockCallCount++,
        ).withDependency<ObserveWalletLockedUseCase>((context) => observeWalletLockedUseCase),
      );
      isLockedStream.add(true);
      await tester.pumpAndSettle();
      expect(onLockCallCount, 1);
      expect(onUnlockCallCount, 0);
    },
  );

  testWidgets(
    'verify onUnlock callback',
    (WidgetTester tester) async {
      int onLockCallCount = 0;
      int onUnlockCallCount = 0;
      await tester.pumpWidgetWithAppWrapper(
        _LockStateTester(
          onLock: () => onLockCallCount++,
          onUnlock: () => onUnlockCallCount++,
        ).withDependency<ObserveWalletLockedUseCase>((context) => observeWalletLockedUseCase),
      );
      isLockedStream.add(true);
      await tester.pumpAndSettle();
      isLockedStream.add(false);
      await tester.pumpAndSettle();
      expect(onLockCallCount, 1);
      expect(onUnlockCallCount, 1);
    },
  );

  testWidgets(
    'verify lock & unlock can trigger multiple times',
    (WidgetTester tester) async {
      int onLockCallCount = 0;
      int onUnlockCallCount = 0;
      await tester.pumpWidgetWithAppWrapper(
        _LockStateTester(
          onLock: () => onLockCallCount++,
          onUnlock: () => onUnlockCallCount++,
        ).withDependency<ObserveWalletLockedUseCase>((context) => observeWalletLockedUseCase),
      );
      isLockedStream.add(true);
      await tester.pumpAndSettle();
      isLockedStream.add(false);
      await tester.pumpAndSettle();
      isLockedStream.add(true);
      await tester.pumpAndSettle();
      isLockedStream.add(false);
      await tester.pumpAndSettle();
      expect(onLockCallCount, 2);
      expect(onUnlockCallCount, 2);
    },
  );

  testWidgets(
    'verify lock does not trigger consecutively',
    (WidgetTester tester) async {
      int onLockCallCount = 0;
      int onUnlockCallCount = 0;
      await tester.pumpWidgetWithAppWrapper(
        _LockStateTester(
          onLock: () => onLockCallCount++,
          onUnlock: () => onUnlockCallCount++,
        ).withDependency<ObserveWalletLockedUseCase>((context) => observeWalletLockedUseCase),
      );
      isLockedStream.add(true);
      await tester.pumpAndSettle();
      isLockedStream.add(true);
      await tester.pumpAndSettle();
      isLockedStream.add(true);
      await tester.pumpAndSettle();
      expect(onLockCallCount, 1);
    },
  );

  testWidgets(
    'verify unlock does not trigger consecutively',
    (WidgetTester tester) async {
      int onLockCallCount = 0;
      int onUnlockCallCount = 0;
      await tester.pumpWidgetWithAppWrapper(
        _LockStateTester(
          onLock: () => onLockCallCount++,
          onUnlock: () => onUnlockCallCount++,
        ).withDependency<ObserveWalletLockedUseCase>((context) => observeWalletLockedUseCase),
      );
      isLockedStream.add(false);
      await tester.pumpAndSettle();
      isLockedStream.add(false);
      await tester.pumpAndSettle();
      isLockedStream.add(false);
      await tester.pumpAndSettle();
      expect(onUnlockCallCount, 1);
    },
  );
}

class _LockStateTester extends StatefulWidget {
  final VoidCallback onLock;
  final VoidCallback onUnlock;

  const _LockStateTester({
    required this.onLock,
    required this.onUnlock,
  });

  @override
  State<_LockStateTester> createState() => _LockStateTesterState();
}

class _LockStateTesterState extends State<_LockStateTester> with LockStateMixin<_LockStateTester> {
  @override
  Future<void> onLock() async => widget.onLock();

  @override
  Future<void> onUnlock() async => widget.onUnlock();

  @override
  Widget build(BuildContext context) => const Placeholder();
}
