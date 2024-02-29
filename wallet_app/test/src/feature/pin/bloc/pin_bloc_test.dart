import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/usecase/network/check_has_internet_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/check_pin_usecase.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  late PinBloc bloc;
  late CheckPinUseCase checkPinUseCase;
  late CheckHasInternetUseCase checkHasInternetUseCase;

  setUp(() {
    // Provide a fallback dummy value for mockito, required here but likely overridden.
    provideDummy<CheckPinResult>(CheckPinResultBlocked());
    checkPinUseCase = MockCheckPinUseCase();
    checkHasInternetUseCase = Mocks.create();
    bloc = PinBloc(checkPinUseCase);
  });

  void triggerValidateFromCleanBloc(PinBloc bloc, CheckPinResult Function() respondWith) {
    when(checkPinUseCase.invoke('100000')).thenAnswer((_) async => respondWith());
    bloc.add(const PinDigitPressed(1));
    bloc.add(const PinDigitPressed(0));
    bloc.add(const PinDigitPressed(0));
    bloc.add(const PinDigitPressed(0));
    bloc.add(const PinDigitPressed(0));
    bloc.add(const PinDigitPressed(0));
  }

  group('Pin entry', () {
    blocTest<PinBloc, PinState>(
      'PinEntryInProgress counter should increase with every pressed digit until and start validating at 6 characters',
      build: () => bloc,
      setUp: () => when(checkPinUseCase.invoke('333333')).thenAnswer((_) async => CheckPinResultOk()),
      act: (bloc) {
        bloc.add(const PinDigitPressed(3));
        bloc.add(const PinDigitPressed(3));
        bloc.add(const PinDigitPressed(3));
        bloc.add(const PinDigitPressed(3));
        bloc.add(const PinDigitPressed(3));
        bloc.add(const PinDigitPressed(3));
      },
      expect: () => [
        const PinEntryInProgress(1),
        const PinEntryInProgress(2),
        const PinEntryInProgress(3),
        const PinEntryInProgress(4),
        const PinEntryInProgress(5),
        const PinValidateInProgress(),
        const PinValidateSuccess(),
      ],
    );

    blocTest<PinBloc, PinState>(
      'PinEntryInProgress counter should decrease when backspace is pressed',
      build: () => bloc,
      act: (bloc) {
        bloc.add(const PinDigitPressed(3));
        bloc.add(const PinDigitPressed(3));
        bloc.add(const PinBackspacePressed());
        bloc.add(const PinBackspacePressed());
      },
      expect: () => [
        const PinEntryInProgress(1),
        const PinEntryInProgress(2),
        const PinEntryInProgress(1, afterBackspacePressed: true),
        const PinEntryInProgress(0, afterBackspacePressed: true),
      ],
    );
  });

  group('Pin validation', () {
    blocTest<PinBloc, PinState>(
      'Verify that WalletInstructionResult.IncorrectPin results in PinValidateFailure with 3 leftover attempts',
      build: () => bloc,
      act: (bloc) => triggerValidateFromCleanBloc(
        bloc,
        () => CheckPinResultIncorrect(leftoverAttempts: 3),
      ),
      skip: 6,
      expect: () => [const PinValidateFailure(leftoverAttempts: 3, isFinalAttempt: false)],
    );
    blocTest<PinBloc, PinState>(
      'Verify that WalletInstructionResult.IncorrectPin results in PinValidateFailure with final attempt',
      build: () => bloc,
      act: (bloc) => triggerValidateFromCleanBloc(
        bloc,
        () => CheckPinResultIncorrect(leftoverAttempts: 1, isFinalAttempt: true),
      ),
      skip: 6,
      expect: () => [const PinValidateFailure(leftoverAttempts: 1, isFinalAttempt: true)],
    );
    blocTest<PinBloc, PinState>(
      'Verify that WalletInstructionResult.Blocked results in PinValidateBlocked',
      build: () => bloc,
      act: (bloc) => triggerValidateFromCleanBloc(
        bloc,
        () => CheckPinResultBlocked(),
      ),
      skip: 6,
      expect: () => [const PinValidateBlocked()],
    );
    blocTest<PinBloc, PinState>(
      'Verify that WalletInstructionResult.Timeout results in PinValidateTimeout',
      build: () => bloc,
      act: (bloc) => triggerValidateFromCleanBloc(
        bloc,
        () => CheckPinResultTimeout(timeoutMillis: 1000),
      ),
      skip: 6,
      expect: () => [isA<PinValidateTimeout>()],
    );
    blocTest<PinBloc, PinState>(
      'Verify that CheckPinResultGenericError results in PinValidateGenericError',
      build: () => bloc,
      act: (bloc) => triggerValidateFromCleanBloc(
        bloc,
        () => throw const CoreGenericError('generic'),
      ),
      skip: 6,
      expect: () => [const PinValidateGenericError(error: CoreGenericError('generic'))],
    );
    blocTest<PinBloc, PinState>(
      'Verify that CheckPinResultServerError results in PinValidateNetworkError with true as hasInternet flag',
      build: () => bloc,
      act: (bloc) => triggerValidateFromCleanBloc(
        bloc,
        () => throw const CoreNetworkError('network'),
      ),
      skip: 6,
      expect: () => [const PinValidateNetworkError(error: CoreNetworkError('network'), hasInternet: true)],
    );
    blocTest<PinBloc, PinState>(
      'Verify that CheckPinResultServerError results in PinValidateNetworkError with false as hasInternet flag',
      build: () => bloc,
      setUp: () => when(checkHasInternetUseCase.invoke()).thenAnswer((realInvocation) async => false),
      act: (bloc) => triggerValidateFromCleanBloc(
        bloc,
        () => throw const CoreNetworkError('network'),
      ),
      skip: 6,
      expect: () => [const PinValidateNetworkError(error: CoreNetworkError('network'), hasInternet: false)],
    );
  });
}
