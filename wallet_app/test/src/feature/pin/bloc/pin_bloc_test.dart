import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/network/network_repository.dart';
import 'package:wallet/src/domain/model/pin/check_pin_result.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/pin/check_pin_usecase.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  late PinBloc bloc;
  late CheckPinUseCase checkPinUseCase;
  late NetworkRepository networkRepository;

  setUp(() {
    // Provide a fallback dummy value for mockito, required here but likely overridden.
    provideDummy<Result<String?>>(const Result.success(null));
    provideDummy<CheckPinResult>(CheckPinResultBlocked());
    checkPinUseCase = MockCheckPinUseCase();
    networkRepository = Mocks.create();
    bloc = PinBloc(checkPinUseCase);
  });

  void triggerValidateFromCleanBloc(PinBloc bloc, Result<String?> Function() respondWith) {
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
      setUp: () => when(checkPinUseCase.invoke('333333')).thenAnswer((_) async => const Result.success(null)),
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
        () => Result.error(IncorrectPinError(CheckPinResultIncorrect(attemptsLeftInRound: 3), sourceError: 'test')),
      ),
      skip: 6,
      expect: () => [const PinValidateFailure(attemptsLeftInRound: 3, isFinalRound: false)],
    );
    blocTest<PinBloc, PinState>(
      'Verify that WalletInstructionResult.IncorrectPin results in PinValidateFailure with final round',
      build: () => bloc,
      act: (bloc) => triggerValidateFromCleanBloc(
        bloc,
        () => Result.error(
          IncorrectPinError(
            CheckPinResultIncorrect(attemptsLeftInRound: 1, isFinalRound: true),
            sourceError: 'test',
          ),
        ),
      ),
      skip: 6,
      expect: () => [const PinValidateFailure(attemptsLeftInRound: 1, isFinalRound: true)],
    );
    blocTest<PinBloc, PinState>(
      'Verify that WalletInstructionResult.Blocked results in PinValidateBlocked',
      build: () => bloc,
      act: (bloc) => triggerValidateFromCleanBloc(
        bloc,
        () => Result.error(IncorrectPinError(CheckPinResultBlocked(), sourceError: 'test')),
      ),
      skip: 6,
      expect: () => [const PinValidateBlocked()],
    );
    blocTest<PinBloc, PinState>(
      'Verify that WalletInstructionResult.Timeout results in PinValidateTimeout',
      build: () => bloc,
      act: (bloc) => triggerValidateFromCleanBloc(
        bloc,
        () => Result.error(IncorrectPinError(CheckPinResultTimeout(timeoutMillis: 1000), sourceError: 'test')),
      ),
      skip: 6,
      expect: () => [isA<PinValidateTimeout>()],
    );
    blocTest<PinBloc, PinState>(
      'Verify that CheckPinResultGenericError results in PinValidateGenericError',
      build: () => bloc,
      act: (bloc) => triggerValidateFromCleanBloc(
        bloc,
        () => const Result.error(GenericError(CoreGenericError('generic'), sourceError: 'test')),
      ),
      skip: 6,
      expect: () => [isA<PinValidateGenericError>()],
    );
    blocTest<PinBloc, PinState>(
      'Verify that CheckPinResultServerError results in PinValidateNetworkError with true as hasInternet flag',
      build: () => bloc,
      act: (bloc) => triggerValidateFromCleanBloc(
        bloc,
        () => const Result.error(NetworkError(hasInternet: true, sourceError: 'test')),
      ),
      skip: 6,
      expect: () => [
        isA<PinValidateNetworkError>().having(
          (error) => error.hasInternet,
          'hasInternet should be true',
          isTrue,
        ),
      ],
    );
    blocTest<PinBloc, PinState>(
      'Verify that CheckPinResultServerError results in PinValidateNetworkError with false as hasInternet flag',
      build: () => bloc,
      setUp: () => when(networkRepository.hasInternet()).thenAnswer((realInvocation) async => false),
      act: (bloc) => triggerValidateFromCleanBloc(
        bloc,
        () => const Result.error(NetworkError(hasInternet: false, sourceError: 'test')),
      ),
      skip: 6,
      expect: () => [
        isA<PinValidateNetworkError>().having(
          (error) => error.hasInternet,
          'hasInternet should be false',
          isFalse,
        ),
      ],
    );
  });
}
