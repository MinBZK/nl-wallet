import 'package:bloc_test/bloc_test.dart';
import 'package:mockito/mockito.dart';
import 'package:test/test.dart';
import 'package:wallet/src/domain/model/pin/pin_validation_error.dart';
import 'package:wallet/src/feature/setup_security/bloc/setup_security_bloc.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  late MockCheckIsValidPinUseCase checkIsValidPinUseCase;
  late MockCreateWalletUseCase createWalletUseCase;
  late MockUnlockWalletWithPinUseCase unlockWalletWithPinUseCase;

  setUp(() {
    checkIsValidPinUseCase = MockCheckIsValidPinUseCase();
    createWalletUseCase = MockCreateWalletUseCase();
    unlockWalletWithPinUseCase = MockUnlockWalletWithPinUseCase();
  });

  blocTest(
    'verify initial state',
    build: () => SetupSecurityBloc(
      checkIsValidPinUseCase,
      createWalletUseCase,
      unlockWalletWithPinUseCase,
    ),
    verify: (bloc) {
      expect(bloc.state, const SetupSecuritySelectPinInProgress(0));
    },
  );

  blocTest(
    'verify state transitions when choosing valid pin',
    build: () => SetupSecurityBloc(
      checkIsValidPinUseCase,
      createWalletUseCase,
      unlockWalletWithPinUseCase,
    ),
    act: (bloc) {
      bloc.add(const PinDigitPressed(9));
      bloc.add(const PinDigitPressed(9));
      bloc.add(const PinDigitPressed(9));
      bloc.add(const PinDigitPressed(8));
      bloc.add(const PinDigitPressed(9));
      bloc.add(const PinDigitPressed(9));
    },
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
    build: () => SetupSecurityBloc(
      checkIsValidPinUseCase,
      createWalletUseCase,
      unlockWalletWithPinUseCase,
    ),
    setUp: () {
      when(checkIsValidPinUseCase.invoke(any)).thenAnswer((_) async => throw PinValidationError.tooFewUniqueDigits);
    },
    act: (bloc) {
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
    },
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
    'verify state transitions when confirming valid pin',
    build: () => SetupSecurityBloc(
      checkIsValidPinUseCase,
      createWalletUseCase,
      unlockWalletWithPinUseCase,
    ),
    act: (bloc) {
      // Choose initial pin
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(1));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      // Confirm pin
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(1));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
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
      SetupSecurityCompleted(),
    ],
  );

  blocTest(
    'verify state transitions when incorrectly confirming valid pin',
    build: () => SetupSecurityBloc(
      checkIsValidPinUseCase,
      createWalletUseCase,
      unlockWalletWithPinUseCase,
    ),
    act: (bloc) {
      // Choose initial pin
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(1));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      // Confirm pin (incorrectly)
      bloc.add(const PinDigitPressed(9));
      bloc.add(const PinDigitPressed(9));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(1));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(9));
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
    build: () => SetupSecurityBloc(
      checkIsValidPinUseCase,
      createWalletUseCase,
      unlockWalletWithPinUseCase,
    ),
    setUp: () {
      when(createWalletUseCase.invoke(any)).thenAnswer((_) async => throw const CoreNetworkError('error'));
    },
    act: (bloc) {
      // Choose initial pin
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(1));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      // Confirm pin
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(1));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
    },
    skip: 11 /* skip pin setup */,
    expect: () => [
      SetupSecurityCreatingWallet(),
      const SetupSecurityNetworkError(error: CoreNetworkError('error'), hasInternet: true),
      isA<SetupSecuritySelectPinInProgress>(),
    ],
  );

  blocTest(
    'verify state transition when wallet creation fails with generic error',
    build: () => SetupSecurityBloc(
      checkIsValidPinUseCase,
      createWalletUseCase,
      unlockWalletWithPinUseCase,
    ),
    setUp: () {
      when(createWalletUseCase.invoke(any)).thenAnswer((_) async => throw const CoreGenericError('generic'));
    },
    act: (bloc) {
      // Choose initial pin
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(1));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      // Confirm pin
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(1));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
    },
    skip: 11 /* skip pin setup */,
    expect: () => [
      SetupSecurityCreatingWallet(),
      const SetupSecurityGenericError(error: CoreGenericError('generic')),
      isA<SetupSecuritySelectPinInProgress>(),
    ],
  );

  blocTest(
    'verify state transition when wallet creation fails due to missing key hardware',
    build: () => SetupSecurityBloc(
      checkIsValidPinUseCase,
      createWalletUseCase,
      unlockWalletWithPinUseCase,
    ),
    setUp: () {
      when(createWalletUseCase.invoke(any))
          .thenAnswer((_) async => throw const CoreHardwareKeyUnsupportedError('hardware_unsupported'));
    },
    act: (bloc) {
      // Choose initial pin
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(1));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      // Confirm pin
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(1));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(0));
    },
    skip: 11 /* skip pin setup */,
    expect: () => [
      SetupSecurityCreatingWallet(),
      const SetupSecurityDeviceIncompatibleError(error: CoreHardwareKeyUnsupportedError('hardware_unsupported')),
      isA<SetupSecuritySelectPinInProgress>(),
    ],
  );
}
