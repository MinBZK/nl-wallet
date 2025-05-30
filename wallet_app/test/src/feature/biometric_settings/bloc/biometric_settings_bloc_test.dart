import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/usecase/biometrics/biometric_authentication_result.dart';
import 'package:wallet/src/domain/usecase/biometrics/get_available_biometrics_usecase.dart';
import 'package:wallet/src/domain/usecase/biometrics/get_supported_biometrics_usecase.dart';
import 'package:wallet/src/domain/usecase/biometrics/is_biometric_login_enabled_usecase.dart';
import 'package:wallet/src/domain/usecase/biometrics/request_biometrics_usecase.dart';
import 'package:wallet/src/domain/usecase/biometrics/set_biometrics_usecase.dart';
import 'package:wallet/src/feature/biometric_settings/bloc/biometric_settings_bloc.dart';

import '../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late GetSupportedBiometricsUseCase getSupportedBiometricsUseCase;
  late GetAvailableBiometricsUseCase getAvailableBiometricsUseCase;
  late SetBiometricsUseCase setBiometricsUseCase;
  late IsBiometricLoginEnabledUseCase isBiometricLoginEnabledUseCase;
  late RequestBiometricsUseCase requestBiometricsUsecase;

  setUp(() {
    provideDummy<Result<BiometricAuthenticationResult>>(const Result.success(BiometricAuthenticationResult.success));

    getSupportedBiometricsUseCase = MockGetSupportedBiometricsUseCase();
    getAvailableBiometricsUseCase = MockGetAvailableBiometricsUseCase();
    setBiometricsUseCase = MockSetBiometricsUseCase();
    isBiometricLoginEnabledUseCase = MockIsBiometricLoginEnabledUseCase();
    requestBiometricsUsecase = MockRequestBiometricsUseCase();
  });

  blocTest(
    'verify initial state',
    build: () => BiometricSettingsBloc(
      getSupportedBiometricsUseCase,
      getAvailableBiometricsUseCase,
      setBiometricsUseCase,
      isBiometricLoginEnabledUseCase,
      requestBiometricsUsecase,
    ),
    verify: (bloc) => expect(bloc.state, BiometricSettingsInitial()),
  );

  blocTest(
    'verify BiometricSettingsLoaded state',
    build: () => BiometricSettingsBloc(
      getSupportedBiometricsUseCase,
      getAvailableBiometricsUseCase,
      setBiometricsUseCase,
      isBiometricLoginEnabledUseCase,
      requestBiometricsUsecase,
    ),
    act: (bloc) => bloc.add(const BiometricLoadTriggered()),
    expect: () => [const BiometricSettingsLoaded(biometricLoginEnabled: false)],
  );

  blocTest(
    'When supported biometrics cant be fetched, emit BiometricSettingsError',
    build: () => BiometricSettingsBloc(
      getSupportedBiometricsUseCase,
      getAvailableBiometricsUseCase,
      setBiometricsUseCase,
      isBiometricLoginEnabledUseCase,
      requestBiometricsUsecase,
    ),
    setUp: () {
      when(getSupportedBiometricsUseCase.invoke()).thenAnswer(
        (_) async => throw GenericError(
          'rawMessage',
          sourceError: Exception(),
        ),
      );
    },
    act: (bloc) => bloc.add(const BiometricLoadTriggered()),
    expect: () => [const BiometricSettingsError()],
  );

  blocTest(
    'verify BiometricSettingsConfirmPin state shows up when user enables biometrics',
    build: () => BiometricSettingsBloc(
      getSupportedBiometricsUseCase,
      getAvailableBiometricsUseCase,
      setBiometricsUseCase,
      isBiometricLoginEnabledUseCase,
      requestBiometricsUsecase,
    ),
    setUp: () {
      when(isBiometricLoginEnabledUseCase.invoke()).thenAnswer((_) async => false);
      when(requestBiometricsUsecase.invoke()).thenAnswer(
        (_) async => const Result.success(BiometricAuthenticationResult.success),
      );
    },
    act: (bloc) async {
      bloc.add(const BiometricLoadTriggered());
      await Future.delayed(const Duration(milliseconds: 5));
      bloc.add(const BiometricUnlockToggled());
      await Future.delayed(const Duration(milliseconds: 5));
    },
    expect: () => [
      const BiometricSettingsLoaded(biometricLoginEnabled: false),
      const BiometricSettingsLoaded(biometricLoginEnabled: true /* we update the ui immediately */),
      const BiometricSettingsConfirmPin(),
    ],
  );

  blocTest(
    'biometrics stays enabled after successful confirmation with pin',
    build: () => BiometricSettingsBloc(
      getSupportedBiometricsUseCase,
      getAvailableBiometricsUseCase,
      setBiometricsUseCase,
      isBiometricLoginEnabledUseCase,
      requestBiometricsUsecase,
    ),
    setUp: () {
      bool biometricsEnabled = false;
      when(isBiometricLoginEnabledUseCase.invoke()).thenAnswer((_) async => biometricsEnabled);
      when(setBiometricsUseCase.invoke(enable: true, authenticateBeforeEnabling: false)).thenAnswer((_) async {
        biometricsEnabled = true;
        return const Result.success(null);
      });
      when(requestBiometricsUsecase.invoke()).thenAnswer(
        (_) async => const Result.success(BiometricAuthenticationResult.success),
      );
    },
    act: (bloc) async {
      bloc.add(const BiometricLoadTriggered());
      await Future.delayed(const Duration(milliseconds: 5));
      bloc.add(const BiometricUnlockToggled());
      await Future.delayed(const Duration(milliseconds: 5));
      bloc.add(const BiometricUnlockEnabledWithPin());
      await Future.delayed(const Duration(milliseconds: 5));
      bloc.add(const BiometricLoadTriggered()); // Reload and make sure update came through
    },
    expect: () => [
      const BiometricSettingsLoaded(biometricLoginEnabled: false),
      const BiometricSettingsLoaded(biometricLoginEnabled: true /* we update the ui immediately */),
      const BiometricSettingsConfirmPin(),
      const BiometricSettingsLoaded(biometricLoginEnabled: true /* successful confirm so enabled stays true */),
    ],
    verify: (bloc) {
      // Verify biometrics are being enabled
      verify(setBiometricsUseCase.invoke(enable: true, authenticateBeforeEnabling: false)).called(1);
    },
  );

  blocTest(
    'biometrics falls back to disabled after unsuccessful confirmation with pin',
    build: () => BiometricSettingsBloc(
      getSupportedBiometricsUseCase,
      getAvailableBiometricsUseCase,
      setBiometricsUseCase,
      isBiometricLoginEnabledUseCase,
      requestBiometricsUsecase,
    ),
    setUp: () {
      when(isBiometricLoginEnabledUseCase.invoke()).thenAnswer((_) async => false);
      when(requestBiometricsUsecase.invoke()).thenAnswer(
        (_) async => const Result.success(BiometricAuthenticationResult.success),
      );
    },
    act: (bloc) async {
      bloc.add(const BiometricLoadTriggered()); // Initial load
      await Future.delayed(const Duration(milliseconds: 5));
      bloc.add(const BiometricUnlockToggled()); // Enable toggle
      await Future.delayed(const Duration(milliseconds: 5));
      bloc.add(const BiometricLoadTriggered()); // Reload after (here unsuccessful) pin confirmation
    },
    expect: () => [
      const BiometricSettingsLoaded(biometricLoginEnabled: false),
      const BiometricSettingsLoaded(biometricLoginEnabled: true /* we update the ui immediately */),
      const BiometricSettingsConfirmPin(),
      const BiometricSettingsLoaded(
        biometricLoginEnabled: false /* unsuccessful confirm so enabled goes back to false */,
      ),
    ],
  );

  blocTest(
    'verify biometrics can be disabled without entering pin',
    build: () => BiometricSettingsBloc(
      getSupportedBiometricsUseCase,
      getAvailableBiometricsUseCase,
      setBiometricsUseCase,
      isBiometricLoginEnabledUseCase,
      requestBiometricsUsecase,
    ),
    setUp: () {
      when(isBiometricLoginEnabledUseCase.invoke()).thenAnswer((_) async => true);
      when(requestBiometricsUsecase.invoke()).thenAnswer(
        (_) async => const Result.success(BiometricAuthenticationResult.success),
      );
    },
    act: (bloc) async {
      bloc.add(const BiometricLoadTriggered());
      await Future.delayed(const Duration(milliseconds: 5));
      bloc.add(const BiometricUnlockToggled());
      await Future.delayed(const Duration(milliseconds: 5));
    },
    expect: () => [
      const BiometricSettingsLoaded(biometricLoginEnabled: true),
      const BiometricSettingsLoaded(biometricLoginEnabled: false),
    ],
    verify: (bloc) {
      verify(setBiometricsUseCase.invoke(enable: false, authenticateBeforeEnabling: false)).called(1);
    },
  );

  blocTest(
    'verify locked out state is triggrered',
    build: () => BiometricSettingsBloc(
      getSupportedBiometricsUseCase,
      getAvailableBiometricsUseCase,
      setBiometricsUseCase,
      isBiometricLoginEnabledUseCase,
      requestBiometricsUsecase,
    ),
    setUp: () {
      when(isBiometricLoginEnabledUseCase.invoke()).thenAnswer((_) async => false);
      when(requestBiometricsUsecase.invoke()).thenAnswer(
        (_) async => const Result.success(BiometricAuthenticationResult.lockedOut),
      );
    },
    act: (bloc) async {
      bloc.add(const BiometricLoadTriggered());
      await Future.delayed(const Duration(milliseconds: 5));
      bloc.add(const BiometricUnlockToggled());
      await Future.delayed(const Duration(milliseconds: 5));
    },
    expect: () => [
      const BiometricSettingsLoaded(biometricLoginEnabled: false),
      const BiometricSettingsLoaded(biometricLoginEnabled: true /* eager enable */),
      const BiometricSettingsLockedOut(),
      const BiometricSettingsLoaded(biometricLoginEnabled: false /* fall back to truth */),
    ],
  );

  blocTest(
    'verify locked out state is setupRequired',
    build: () => BiometricSettingsBloc(
      getSupportedBiometricsUseCase,
      getAvailableBiometricsUseCase,
      setBiometricsUseCase,
      isBiometricLoginEnabledUseCase,
      requestBiometricsUsecase,
    ),
    setUp: () {
      when(isBiometricLoginEnabledUseCase.invoke()).thenAnswer((_) async => false);
      when(requestBiometricsUsecase.invoke()).thenAnswer(
        (_) async => const Result.success(BiometricAuthenticationResult.setupRequired),
      );
    },
    act: (bloc) async {
      bloc.add(const BiometricLoadTriggered());
      await Future.delayed(const Duration(milliseconds: 5));
      bloc.add(const BiometricUnlockToggled());
      await Future.delayed(const Duration(milliseconds: 5));
    },
    expect: () => [
      const BiometricSettingsLoaded(biometricLoginEnabled: false),
      const BiometricSettingsLoaded(biometricLoginEnabled: true /* eager enable */),
      const BiometricSettingsSetupRequired(),
      const BiometricSettingsLoaded(biometricLoginEnabled: false /* fall back to truth */),
    ],
  );
}
