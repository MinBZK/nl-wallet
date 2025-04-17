import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/wallet/wallet_repository.dart';
import 'package:wallet/src/domain/usecase/app/check_is_app_initialized_usecase.dart';
import 'package:wallet/src/domain/usecase/biometrics/biometrics.dart';
import 'package:wallet/src/domain/usecase/biometrics/is_biometric_login_enabled_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/unlock_wallet_with_pin_usecase.dart';
import 'package:wallet/src/feature/biometric_settings/biometric_settings_screen.dart';
import 'package:wallet/src/feature/biometric_settings/bloc/biometric_settings_bloc.dart';
import 'package:wallet/src/util/manager/biometric_unlock_manager.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.dart';
import '../../test_util/golden_utils.dart';

class MockBiometricSettingsBloc extends MockBloc<BiometricSettingsEvent, BiometricSettingsState>
    implements BiometricSettingsBloc {
  @override
  Biometrics supportedBiometrics = Biometrics.none;
}

void main() {
  setUp(() {
    provideDummy<Result<String?>>(Result.success(''));
  });

  group('goldens', () {
    testGoldens('Biometrics face loaded light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const BiometricSettingScreen().withState<BiometricSettingsBloc, BiometricSettingsState>(
          MockBiometricSettingsBloc()..supportedBiometrics = Biometrics.face,
          const BiometricSettingsLoaded(biometricLoginEnabled: true),
        ),
      );
      await screenMatchesGolden('face_loaded.light');
    });

    testGoldens('Biometrics finger loaded light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const BiometricSettingScreen().withState<BiometricSettingsBloc, BiometricSettingsState>(
          MockBiometricSettingsBloc()..supportedBiometrics = Biometrics.fingerprint,
          const BiometricSettingsLoaded(biometricLoginEnabled: false),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('finger_loaded.dark');
    });

    testGoldens('Biometrics initial light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const BiometricSettingScreen().withState<BiometricSettingsBloc, BiometricSettingsState>(
          MockBiometricSettingsBloc()..supportedBiometrics = Biometrics.fingerprint,
          BiometricSettingsInitial(),
        ),
      );
      await screenMatchesGolden('initial.light');
    });

    testGoldens('Biometrics some loaded dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const BiometricSettingScreen().withState<BiometricSettingsBloc, BiometricSettingsState>(
          MockBiometricSettingsBloc()..supportedBiometrics = Biometrics.some,
          const BiometricSettingsLoaded(biometricLoginEnabled: true),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('some_loaded.dark');
    });

    testGoldens('Biometrics setup required light', (tester) async {
      final bloc = MockBiometricSettingsBloc()..supportedBiometrics = Biometrics.some;

      whenListen(
        bloc,
        Stream<BiometricSettingsState>.value(const BiometricSettingsSetupRequired()),
        initialState: const BiometricSettingsLoaded(biometricLoginEnabled: false),
      );

      await tester.pumpWidgetWithAppWrapper(
        const BiometricSettingScreen(),
        providers: [RepositoryProvider<BiometricSettingsBloc>(create: (c) => bloc)],
      );

      await screenMatchesGolden('setup_required.light');
    });
  });

  testGoldens('Biometrics locked out light', (tester) async {
    final bloc = MockBiometricSettingsBloc()..supportedBiometrics = Biometrics.some;

    whenListen(
      bloc,
      Stream<BiometricSettingsState>.value(const BiometricSettingsLockedOut()),
      initialState: const BiometricSettingsLoaded(biometricLoginEnabled: false),
    );

    await tester.pumpWidgetWithAppWrapper(
      const BiometricSettingScreen(),
      providers: [RepositoryProvider<BiometricSettingsBloc>(create: (c) => bloc)],
    );

    await screenMatchesGolden('locked_out.light');
  });

  testGoldens('Biometrics confirm pin light', (tester) async {
    final bloc = MockBiometricSettingsBloc()..supportedBiometrics = Biometrics.face;
    final mockUnlockUseCase = MockUnlockWalletWithPinUseCase();

    whenListen(
      bloc,
      Stream<BiometricSettingsState>.value(const BiometricSettingsConfirmPin()),
      initialState: const BiometricSettingsLoaded(biometricLoginEnabled: false),
    );

    await tester.pumpWidgetWithAppWrapper(
      const BiometricSettingScreen(),
      providers: [
        RepositoryProvider<BiometricSettingsBloc>(create: (c) => bloc),
        RepositoryProvider<WalletRepository>(
          create: (c) {
            final mock = MockWalletRepository();
            when(mock.isLockedStream).thenAnswer((_) => Stream.value(false));
            return mock;
          },
        ),
        RepositoryProvider<IsWalletInitializedUseCase>(
          create: (c) {
            final mock = MockIsWalletInitializedUseCase();
            when(mock.invoke()).thenAnswer((_) async => true);
            return mock;
          },
        ),
        RepositoryProvider<IsBiometricLoginEnabledUseCase>(
          create: (c) {
            final mock = MockIsBiometricLoginEnabledUseCase();
            when(mock.invoke()).thenAnswer((_) async => true);
            return mock;
          },
        ),
        RepositoryProvider<BiometricUnlockManager>(create: (c) => MockBiometricUnlockManager()),
        RepositoryProvider<UnlockWalletWithPinUseCase>(create: (c) => mockUnlockUseCase),
        RepositoryProvider<CheckPinUseCase>(create: (c) => mockUnlockUseCase),
      ],
    );

    await screenMatchesGolden('confirm_pin.light');
  });

  testGoldens('Biometrics pin confirmed light', (tester) async {
    final bloc = MockBiometricSettingsBloc()..supportedBiometrics = Biometrics.fingerprint;
    final mockUnlockUseCase = MockUnlockWalletWithPinUseCase();

    whenListen(
      bloc,
      Stream<BiometricSettingsState>.value(const BiometricSettingsConfirmPin()),
      initialState: const BiometricSettingsLoaded(biometricLoginEnabled: false),
    );

    await tester.pumpWidgetWithAppWrapper(
      const BiometricSettingScreen(),
      providers: [
        RepositoryProvider<BiometricSettingsBloc>(create: (c) => bloc),
        RepositoryProvider<WalletRepository>(
          create: (c) {
            final mock = MockWalletRepository();
            when(mock.isLockedStream).thenAnswer((_) => Stream.value(false));
            return mock;
          },
        ),
        RepositoryProvider<IsWalletInitializedUseCase>(
          create: (c) {
            final mock = MockIsWalletInitializedUseCase();
            when(mock.invoke()).thenAnswer((_) async => true);
            return mock;
          },
        ),
        RepositoryProvider<IsBiometricLoginEnabledUseCase>(
          create: (c) {
            final mock = MockIsBiometricLoginEnabledUseCase();
            when(mock.invoke()).thenAnswer((_) async => true);
            return mock;
          },
        ),
        RepositoryProvider<BiometricUnlockManager>(create: (c) => MockBiometricUnlockManager()),
        RepositoryProvider<UnlockWalletWithPinUseCase>(create: (c) => mockUnlockUseCase),
        RepositoryProvider<CheckPinUseCase>(create: (c) => mockUnlockUseCase),
      ],
    );

    /// Enter a pin to enable biometrics
    await tester.tap(find.byKey(const Key('keyboardDigitKey#1')));
    await tester.tap(find.byKey(const Key('keyboardDigitKey#1')));
    await tester.tap(find.byKey(const Key('keyboardDigitKey#1')));
    await tester.tap(find.byKey(const Key('keyboardDigitKey#3')));
    await tester.tap(find.byKey(const Key('keyboardDigitKey#3')));
    await tester.tap(find.byKey(const Key('keyboardDigitKey#3')));

    await tester.pumpAndSettle();

    await screenMatchesGolden('pin_confirmed.light');
  });
}
