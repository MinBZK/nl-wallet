import 'dart:ui';

import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/feature/common/widget/utility/auto_biometric_unlock_trigger.dart';
import 'package:wallet/src/util/manager/biometric_unlock_manager.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/wallet_mocks.dart';

void main() {
  late MockBiometricUnlockManager mockBiometricUnlockManager;

  setUp(() {
    mockBiometricUnlockManager = MockBiometricUnlockManager();
  });

  group('widgets', () {
    testWidgets('trigger is not fired when manager reports false', (tester) async {
      when(mockBiometricUnlockManager.shouldTriggerUnlock).thenReturn(false);
      when(mockBiometricUnlockManager.getAndSetShouldTriggerUnlock(updatedValue: anyNamed('updatedValue')))
          .thenReturn(false);

      bool isFired = false;
      await tester.pumpWidgetWithAppWrapper(
        AutoBiometricUnlockTrigger(onTriggerBiometricUnlock: (c) => isFired = true),
        providers: [
          RepositoryProvider<BiometricUnlockManager>(create: (c) => mockBiometricUnlockManager),
        ],
      );

      expect(isFired, isFalse);
    });

    testWidgets('trigger is fired when manager reports true', (tester) async {
      when(mockBiometricUnlockManager.shouldTriggerUnlock).thenReturn(true);
      when(mockBiometricUnlockManager.getAndSetShouldTriggerUnlock(updatedValue: anyNamed('updatedValue')))
          .thenReturn(true);

      bool isFired = false;
      await tester.pumpWidgetWithAppWrapper(
        AutoBiometricUnlockTrigger(onTriggerBiometricUnlock: (c) => isFired = true),
        providers: [
          RepositoryProvider<BiometricUnlockManager>(create: (c) => mockBiometricUnlockManager),
        ],
      );

      expect(isFired, isTrue);
    });

    testWidgets('trigger is re-checked and fired onResume', (tester) async {
      when(mockBiometricUnlockManager.shouldTriggerUnlock).thenReturn(false);
      when(mockBiometricUnlockManager.getAndSetShouldTriggerUnlock(updatedValue: anyNamed('updatedValue')))
          .thenReturn(false);

      bool isFired = false;
      await tester.pumpWidgetWithAppWrapper(
        AutoBiometricUnlockTrigger(onTriggerBiometricUnlock: (c) => isFired = true),
        providers: [
          RepositoryProvider<BiometricUnlockManager>(create: (c) => mockBiometricUnlockManager),
        ],
      );
      expect(isFired, isFalse);

      // Update trigger
      when(mockBiometricUnlockManager.shouldTriggerUnlock).thenReturn(true);
      when(mockBiometricUnlockManager.getAndSetShouldTriggerUnlock(updatedValue: anyNamed('updatedValue')))
          .thenReturn(true);

      // Verify nothing happens yet
      expect(isFired, isFalse);

      // Fake resume event
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);

      // Verify trigger is fired
      await tester.pumpAndSettle(); // Make sure any delays are processed (i.e. kTriggerDelay)
      expect(isFired, isTrue);
    });
  });
}
