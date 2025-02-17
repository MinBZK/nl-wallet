import 'package:bloc_test/bloc_test.dart';
import 'package:mockito/mockito.dart';
import 'package:test/test.dart';
import 'package:wallet/src/domain/model/pin/pin_validation_error.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/usecase/biometrics/get_available_biometrics_usecase.dart';
import 'package:wallet/src/feature/setup_security/bloc/setup_security_bloc.dart';
import 'package:wallet/src/wallet_constants.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  late MockCheckIsValidPinUseCase checkIsValidPinUseCase;
  late MockCreateWalletUseCase createWalletUseCase;
  late MockGetAvailableBiometricsUseCase getAvailableBiometricsUseCase;
  late MockSetBiometricsUseCase setBiometricsUseCase;

  setUp(() {
    checkIsValidPinUseCase = MockCheckIsValidPinUseCase();
    createWalletUseCase = MockCreateWalletUseCase();
    getAvailableBiometricsUseCase = MockGetAvailableBiometricsUseCase();
    setBiometricsUseCase = MockSetBiometricsUseCase();
  });

  SetupSecurityBloc buildBloc() => SetupSecurityBloc(
        checkIsValidPinUseCase,
        createWalletUseCase,
        getAvailableBiometricsUseCase,
        setBiometricsUseCase,
      );

  blocTest(
    'verify initial state',
    build: buildBloc,
    verify: (bloc) {
      expect(bloc.state, const SetupSecuritySelectPinInProgress(0));
    },
  );

  blocTest(
    'verify state transitions when choosing valid pin',
    build: buildBloc,
    act: (bloc) => bloc.enterPin('999899'),
    expect: () => [
      const SetupSecuritySelectPinInProgress(1),
      const SetupSecuritySelectPinInProgress(2),
      const SetupSecuritySelectPinInProgress(3),
      const SetupSecuritySelectPinInProgress(4),
      const SetupSecuritySelectPinInProgress(5),
      const SetupSecurityPinConfirmationInProgress(0),
    ],
  );

  blocTest(
    'verify state transitions when choosing invalid pin',
    build: buildBloc,
    setUp: () {
      when(checkIsValidPinUseCase.invoke(any)).thenAnswer(
        (_) async => const Result.error(ValidatePinError(PinValidationError.tooFewUniqueDigits, sourceError: 'test')),
      );
    },
    act: (bloc) => bloc.enterPin('000000'),
    expect: () => [
      const SetupSecuritySelectPinInProgress(1),
      const SetupSecuritySelectPinInProgress(2),
      const SetupSecuritySelectPinInProgress(3),
      const SetupSecuritySelectPinInProgress(4),
      const SetupSecuritySelectPinInProgress(5),
      const SetupSecuritySelectPinFailed(reason: PinValidationError.tooFewUniqueDigits),
    ],
  );

  blocTest(
    'verify state transitions when confirming valid pin and device does not support biometrics',
    build: buildBloc,
    setUp: () async {
      when(getAvailableBiometricsUseCase.invoke()).thenAnswer((_) async => Biometrics.none);
    },
    act: (bloc) {
      // Choose initial pin
      bloc.enterPin('000100');
      // Confirm pin
      bloc.enterPin('000100');
    },
    skip: 5 /* skip initial pin state changes */,
    expect: () => [
      const SetupSecurityPinConfirmationInProgress(0),
      const SetupSecurityPinConfirmationInProgress(1),
      const SetupSecurityPinConfirmationInProgress(2),
      const SetupSecurityPinConfirmationInProgress(3),
      const SetupSecurityPinConfirmationInProgress(4),
      const SetupSecurityPinConfirmationInProgress(5),
      SetupSecurityCreatingWallet(),
      const SetupSecurityCompleted(),
    ],
  );

  blocTest(
    'verify state transitions when incorrectly confirming valid pin',
    build: buildBloc,
    act: (bloc) {
      // Choose initial pin
      bloc.enterPin('000100');
      // Confirm pin (incorrectly)
      bloc.enterPin('990109');
    },
    skip: 5 /* skip initial pin state changes */,
    expect: () => [
      const SetupSecurityPinConfirmationInProgress(0),
      const SetupSecurityPinConfirmationInProgress(1),
      const SetupSecurityPinConfirmationInProgress(2),
      const SetupSecurityPinConfirmationInProgress(3),
      const SetupSecurityPinConfirmationInProgress(4),
      const SetupSecurityPinConfirmationInProgress(5),
      const SetupSecurityPinConfirmationFailed(retryAllowed: true),
    ],
  );

  blocTest(
    'verify state transition when wallet creation fails with network error',
    build: buildBloc,
    setUp: () {
      when(createWalletUseCase.invoke(any))
          .thenAnswer((_) async => const Result.error(NetworkError(hasInternet: true, sourceError: 'test')));
    },
    act: (bloc) {
      // Choose initial pin
      bloc.enterPin('000100');
      // Confirm pin
      bloc.enterPin('000100');
    },
    skip: 11 /* skip pin setup */,
    expect: () => [
      SetupSecurityCreatingWallet(),
      isA<SetupSecurityNetworkError>().having((e) => e.hasInternet, 'hasInternet', true),
      isA<SetupSecuritySelectPinInProgress>(),
    ],
  );

  blocTest(
    'verify state transition when wallet creation fails with generic error',
    build: buildBloc,
    setUp: () {
      when(createWalletUseCase.invoke(any))
          .thenAnswer((_) async => const Result.error(GenericError('generic', sourceError: 'test')));
    },
    act: (bloc) {
      // Choose initial pin
      bloc.enterPin('000100');
      // Confirm pin
      bloc.enterPin('000100');
    },
    skip: 11 /* skip pin setup */,
    expect: () => [
      SetupSecurityCreatingWallet(),
      isA<SetupSecurityGenericError>(),
      isA<SetupSecuritySelectPinInProgress>(),
    ],
  );

  blocTest(
    'verify state transition when wallet creation fails due to missing key hardware',
    build: buildBloc,
    setUp: () {
      when(createWalletUseCase.invoke(any))
          .thenAnswer((_) async => const Result.error(HardwareUnsupportedError(sourceError: 'test')));
    },
    act: (bloc) {
      // Choose initial pin
      bloc.enterPin('000100');
      // Confirm pin
      bloc.enterPin('000100');
    },
    skip: 11 /* skip pin setup */,
    expect: () => [
      SetupSecurityCreatingWallet(),
      isA<SetupSecurityDeviceIncompatibleError>(),
      isA<SetupSecuritySelectPinInProgress>(),
    ],
  );

  blocTest(
    'verify state transitions when confirming valid pin and device supports biometrics',
    build: buildBloc,
    setUp: () async {
      when(getAvailableBiometricsUseCase.invoke()).thenAnswer((_) async => Biometrics.some);
    },
    act: (bloc) {
      // Choose initial pin
      bloc.enterPin('000100');
      // Confirm pin
      bloc.enterPin('000100');
    },
    skip: 11 /* skip pin & pin confirmation state changes */,
    expect: () => [
      SetupSecurityCreatingWallet(),
      const SetupSecurityConfigureBiometrics(biometrics: Biometrics.some),
    ],
  );

  blocTest(
    'when enabling biometrics, the set biometrics usecase is invoked',
    build: buildBloc,
    setUp: () async {
      when(getAvailableBiometricsUseCase.invoke()).thenAnswer((_) async => Biometrics.some);
      when(
        setBiometricsUseCase.invoke(
          enable: anyNamed('enable'),
          authenticateBeforeEnabling: anyNamed('authenticateBeforeEnabling'),
        ),
      ).thenAnswer((_) async => const Result.success(null));
    },
    act: (bloc) async {
      // Choose initial pin
      bloc.enterPin('000100');
      // Confirm pin
      bloc.enterPin('000100');
      // Enable biometrics
      await Future.delayed(const Duration(milliseconds: 10));
      bloc.add(EnableBiometricsPressed());
    },
    skip: 11 /* skip pin & pin confirmation state changes */,
    expect: () => [
      SetupSecurityCreatingWallet(),
      const SetupSecurityConfigureBiometrics(biometrics: Biometrics.some),
      const SetupSecurityCompleted(enabledBiometrics: Biometrics.some),
    ],
    verify: (bloc) => verify(setBiometricsUseCase.invoke(enable: true, authenticateBeforeEnabling: true)).called(1),
  );

  blocTest(
    'when skipping biometric setup, the set biometrics usecase is not invoked',
    build: buildBloc,
    setUp: () async {
      when(getAvailableBiometricsUseCase.invoke()).thenAnswer((_) async => Biometrics.some);
    },
    act: (bloc) async {
      // Choose initial pin
      bloc.enterPin('000100');
      // Confirm pin
      bloc.enterPin('000100');
      // Skip biometric setup
      await Future.delayed(const Duration(milliseconds: 10));
      bloc.add(SkipBiometricsPressed());
    },
    skip: 11 /* skip pin & pin confirmation state changes */,
    expect: () => [
      SetupSecurityCreatingWallet(),
      const SetupSecurityConfigureBiometrics(biometrics: Biometrics.some),
      const SetupSecurityCompleted(),
    ],
    verify: (bloc) => verifyZeroInteractions(setBiometricsUseCase),
  );
}

extension _SetupSecurityBlocExtensions on SetupSecurityBloc {
  void enterPin(String pin) {
    assert(pin.length == kPinDigits, 'Invalid pin');
    pin.split('').map(int.parse).forEach((digit) {
      add(PinDigitPressed(digit));
    });
  }
}
