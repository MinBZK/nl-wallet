import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/pin/pin_validation_error.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/feature/change_pin/bloc/change_pin_bloc.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  late MockCheckIsValidPinUseCase checkIsValidPinUseCase;
  late MockChangePinUseCase changePinUseCase;

  setUp(() {
    checkIsValidPinUseCase = MockCheckIsValidPinUseCase();
    changePinUseCase = MockChangePinUseCase();
  });

  blocTest(
    'verify initial state',
    build: () => ChangePinBloc(Mocks.create(), Mocks.create()),
    verify: (bloc) => expect(bloc.state, isA<ChangePinInitial>()),
  );

  blocTest(
    'verify providing current pin emits ChangePinSelectNewPinInProgress',
    build: () => ChangePinBloc(checkIsValidPinUseCase, changePinUseCase),
    act: (bloc) => bloc.add(const ChangePinCurrentPinValidated('000111')),
    expect: () => [const ChangePinSelectNewPinInProgress(0)],
  );

  blocTest(
    'verify providing invalid new pin emits ChangePinSelectNewPinFailed',
    build: () => ChangePinBloc(checkIsValidPinUseCase, changePinUseCase),
    setUp: () => when(checkIsValidPinUseCase.invoke(any)).thenAnswer(
      (_) async => const Result.error(
        ValidatePinError(PinValidationError.sequentialDigits, sourceError: 'test'),
      ),
    ),
    act: (bloc) {
      bloc.add(const ChangePinCurrentPinValidated('123456'));
      bloc.provideInvalidPin();
    },
    expect: () => [
      const ChangePinSelectNewPinInProgress(0),
      const ChangePinSelectNewPinInProgress(1),
      const ChangePinSelectNewPinInProgress(2),
      const ChangePinSelectNewPinInProgress(3),
      const ChangePinSelectNewPinInProgress(4),
      const ChangePinSelectNewPinInProgress(5),
      const ChangePinSelectNewPinFailed(reason: PinValidationError.sequentialDigits),
    ],
  );

  blocTest(
    'verify providing valid new pin emits ChangePinSelectNewPinFailed',
    build: () => ChangePinBloc(checkIsValidPinUseCase, changePinUseCase),
    setUp: () => when(checkIsValidPinUseCase.invoke(any)).thenAnswer((_) async => const Result.success(null)),
    act: (bloc) {
      bloc.add(const ChangePinCurrentPinValidated('000111'));
      bloc.provideValidPin();
    },
    expect: () => [
      const ChangePinSelectNewPinInProgress(0),
      const ChangePinSelectNewPinInProgress(1),
      const ChangePinSelectNewPinInProgress(2),
      const ChangePinSelectNewPinInProgress(3),
      const ChangePinSelectNewPinInProgress(4),
      const ChangePinSelectNewPinInProgress(5),
      const ChangePinConfirmNewPinInProgress(0),
    ],
  );

  blocTest(
    'verify confirming new pin with a different pin results in ChangePinConfirmNewPinFailed',
    build: () => ChangePinBloc(checkIsValidPinUseCase, changePinUseCase),
    setUp: () => when(checkIsValidPinUseCase.invoke(any)).thenAnswer((_) async => const Result.success(null)),
    act: (bloc) {
      bloc.add(const ChangePinCurrentPinValidated('000111'));
      bloc.provideValidPin();
      bloc.provideInvalidPin();
    },
    skip: 6 /* skip setting up new pin */,
    expect: () => [
      const ChangePinConfirmNewPinInProgress(0),
      const ChangePinConfirmNewPinInProgress(1),
      const ChangePinConfirmNewPinInProgress(2),
      const ChangePinConfirmNewPinInProgress(3),
      const ChangePinConfirmNewPinInProgress(4),
      const ChangePinConfirmNewPinInProgress(5),
      const ChangePinConfirmNewPinFailed(retryAllowed: true),
    ],
  );

  blocTest(
    'verify retying the confirm new pin step is only allowed once',
    build: () => ChangePinBloc(checkIsValidPinUseCase, changePinUseCase),
    setUp: () => when(checkIsValidPinUseCase.invoke(any)).thenAnswer((_) async => const Result.success(null)),
    act: (bloc) {
      bloc.add(const ChangePinCurrentPinValidated('000111'));
      bloc.provideValidPin();
      bloc.provideInvalidPin();
      bloc.provideInvalidPin();
    },
    skip: 12 /* skip setting up new pin and first confirmation round */,
    expect: () => [
      const ChangePinConfirmNewPinFailed(retryAllowed: true),
      const ChangePinConfirmNewPinInProgress(1),
      const ChangePinConfirmNewPinInProgress(2),
      const ChangePinConfirmNewPinInProgress(3),
      const ChangePinConfirmNewPinInProgress(4),
      const ChangePinConfirmNewPinInProgress(5),
      const ChangePinConfirmNewPinFailed(retryAllowed: false),
    ],
  );

  blocTest(
    'verify successful pin change leads to updating and completed state',
    build: () => ChangePinBloc(checkIsValidPinUseCase, changePinUseCase),
    setUp: () => when(checkIsValidPinUseCase.invoke(any)).thenAnswer((_) async => const Result.success(null)),
    act: (bloc) {
      bloc.add(const ChangePinCurrentPinValidated('000111'));
      bloc.provideValidPin();
      bloc.provideValidPin();
    },
    skip: 12 /* skip setting up new pin and confirming pin */,
    expect: () => [
      ChangePinUpdating(),
      ChangePinCompleted(),
    ],
  );

  blocTest(
    'verify unsuccessful pin change (network error) leads to ChangePinNetworkError followed by a reset to ChangePinInitial',
    build: () => ChangePinBloc(checkIsValidPinUseCase, changePinUseCase),
    setUp: () {
      when(checkIsValidPinUseCase.invoke(any)).thenAnswer((_) async => const Result.success(null));
      when(changePinUseCase.invoke(any, any))
          .thenAnswer((_) async => const Result.error(NetworkError(hasInternet: true, sourceError: 'test')));
    },
    act: (bloc) {
      bloc.add(const ChangePinCurrentPinValidated('000111'));
      bloc.provideValidPin();
      bloc.provideValidPin();
    },
    skip: 12 /* skip setting up new pin and confirming pin */,
    expect: () => [
      ChangePinUpdating(),
      const ChangePinNetworkError(error: NetworkError(hasInternet: true, sourceError: 'test'), hasInternet: true),
      const ChangePinInitial(didGoBack: true),
    ],
  );

  blocTest(
    'verify unsuccessful pin change (generic error) leads to ChangePinGenericError followed by a reset to ChangePinInitial',
    build: () => ChangePinBloc(checkIsValidPinUseCase, changePinUseCase),
    setUp: () {
      when(checkIsValidPinUseCase.invoke(any)).thenAnswer((_) async => const Result.success(null));
      when(changePinUseCase.invoke(any, any))
          .thenAnswer((_) async => const Result.error(GenericError('', sourceError: 'test')));
    },
    act: (bloc) {
      bloc.add(const ChangePinCurrentPinValidated('000111'));
      bloc.provideValidPin();
      bloc.provideValidPin();
    },
    skip: 12 /* skip setting up new pin and confirming pin */,
    expect: () => [
      ChangePinUpdating(),
      isA<ChangePinGenericError>(),
      const ChangePinInitial(didGoBack: true),
    ],
  );

  blocTest(
    'verify pressing back from new pin setup returns to ChangePinInitial',
    build: () => ChangePinBloc(checkIsValidPinUseCase, changePinUseCase),
    setUp: () => when(checkIsValidPinUseCase.invoke(any)).thenAnswer((_) async => const Result.success(null)),
    act: (bloc) {
      bloc.add(const ChangePinCurrentPinValidated('000111'));
      bloc.add(const PinDigitPressed(0));
      bloc.add(ChangePinBackPressed());
    },
    expect: () => [
      const ChangePinSelectNewPinInProgress(0),
      const ChangePinSelectNewPinInProgress(1),
      const ChangePinInitial(didGoBack: true),
    ],
  );

  blocTest(
    'verify pressing back from new pin confirmation returns to ChangePinSelectNewPinInProgress',
    build: () => ChangePinBloc(checkIsValidPinUseCase, changePinUseCase),
    setUp: () => when(checkIsValidPinUseCase.invoke(any)).thenAnswer((_) async => const Result.success(null)),
    act: (bloc) {
      bloc.add(const ChangePinCurrentPinValidated('000111'));
      bloc.provideValidPin();
      bloc.add(const PinDigitPressed(0));
      bloc.add(ChangePinBackPressed());
    },
    skip: 6 /* skip setting up the new pin */,
    expect: () => [
      const ChangePinConfirmNewPinInProgress(0),
      const ChangePinConfirmNewPinInProgress(1),
      const ChangePinSelectNewPinInProgress(0, didGoBack: true),
    ],
  );

  blocTest(
    'verify backspace key removes the last entered digit while entering new pin',
    build: () => ChangePinBloc(checkIsValidPinUseCase, changePinUseCase),
    setUp: () => when(checkIsValidPinUseCase.invoke(any)).thenAnswer((_) async => const Result.success(null)),
    act: (bloc) {
      bloc.add(const ChangePinCurrentPinValidated('000111'));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(1));
      bloc.add(const PinDigitPressed(2));
      bloc.add(PinBackspacePressed());
      bloc.add(PinBackspacePressed());
    },
    expect: () => [
      const ChangePinSelectNewPinInProgress(0),
      const ChangePinSelectNewPinInProgress(1),
      const ChangePinSelectNewPinInProgress(2),
      const ChangePinSelectNewPinInProgress(3),
      const ChangePinSelectNewPinInProgress(2, afterBackspacePressed: true),
      const ChangePinSelectNewPinInProgress(1, afterBackspacePressed: true),
    ],
  );

  blocTest(
    'verify backspace key removes the last entered digit while entering new pin',
    build: () => ChangePinBloc(checkIsValidPinUseCase, changePinUseCase),
    setUp: () => when(checkIsValidPinUseCase.invoke(any)).thenAnswer((_) async => const Result.success(null)),
    act: (bloc) {
      bloc.add(const ChangePinCurrentPinValidated('000111'));
      bloc.provideValidPin();
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(1));
      bloc.add(const PinDigitPressed(2));
      bloc.add(PinBackspacePressed());
      bloc.add(PinBackspacePressed());
    },
    skip: 6 /* skip setting up the new pin */,
    expect: () => [
      const ChangePinConfirmNewPinInProgress(0),
      const ChangePinConfirmNewPinInProgress(1),
      const ChangePinConfirmNewPinInProgress(2),
      const ChangePinConfirmNewPinInProgress(3),
      const ChangePinConfirmNewPinInProgress(2, afterBackspacePressed: true),
      const ChangePinConfirmNewPinInProgress(1, afterBackspacePressed: true),
    ],
  );

  blocTest(
    'verify holding backspace key removes all entered digits while entering new pin',
    build: () => ChangePinBloc(checkIsValidPinUseCase, changePinUseCase),
    setUp: () => when(checkIsValidPinUseCase.invoke(any)).thenAnswer((_) async => const Result.success(null)),
    act: (bloc) {
      bloc.add(const ChangePinCurrentPinValidated('000111'));
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(1));
      bloc.add(const PinDigitPressed(2));
      bloc.add(const PinDigitPressed(3));
      bloc.add(PinClearPressed());
    },
    expect: () => [
      const ChangePinSelectNewPinInProgress(0),
      const ChangePinSelectNewPinInProgress(1),
      const ChangePinSelectNewPinInProgress(2),
      const ChangePinSelectNewPinInProgress(3),
      const ChangePinSelectNewPinInProgress(4),
      const ChangePinSelectNewPinInProgress(0, afterBackspacePressed: true),
    ],
  );

  blocTest(
    'verify holding backspace key removes all entered digits while confirming new pin',
    build: () => ChangePinBloc(checkIsValidPinUseCase, changePinUseCase),
    setUp: () => when(checkIsValidPinUseCase.invoke(any)).thenAnswer((_) async => const Result.success(null)),
    act: (bloc) {
      bloc.add(const ChangePinCurrentPinValidated('000111'));
      bloc.provideValidPin();
      bloc.add(const PinDigitPressed(0));
      bloc.add(const PinDigitPressed(1));
      bloc.add(const PinDigitPressed(2));
      bloc.add(const PinDigitPressed(3));
      bloc.add(PinClearPressed());
    },
    skip: 6 /* skip setting up the new pin */,
    expect: () => [
      const ChangePinConfirmNewPinInProgress(0),
      const ChangePinConfirmNewPinInProgress(1),
      const ChangePinConfirmNewPinInProgress(2),
      const ChangePinConfirmNewPinInProgress(3),
      const ChangePinConfirmNewPinInProgress(4),
      const ChangePinConfirmNewPinInProgress(0, afterBackspacePressed: true),
    ],
  );
}

extension _ChangePinBlocTestExtensions on ChangePinBloc {
  void provideValidPin() {
    add(const PinDigitPressed(0));
    add(const PinDigitPressed(1));
    add(const PinDigitPressed(0));
    add(const PinDigitPressed(2));
    add(const PinDigitPressed(0));
    add(const PinDigitPressed(3));
  }

  void provideInvalidPin() {
    add(const PinDigitPressed(0));
    add(const PinDigitPressed(0));
    add(const PinDigitPressed(0));
    add(const PinDigitPressed(0));
    add(const PinDigitPressed(0));
    add(const PinDigitPressed(0));
  }
}
