import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/pin/pin_validation_error.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/feature/recover_pin/bloc/recover_pin_bloc.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  late MockCreatePinRecoveryRedirectUriUseCase createPinRecoveryRedirectUriUseCase;
  late MockCheckIsValidPinUseCase checkIsValidPinUseCase;
  late MockContinuePinRecoveryUseCase continuePinRecoveryUseCase;
  late MockCancelPinRecoveryUseCase cancelPinRecoveryUseCase;
  late MockCompletePinRecoveryUseCase completePinRecoveryUseCase;

  setUp(() {
    createPinRecoveryRedirectUriUseCase = MockCreatePinRecoveryRedirectUriUseCase();
    checkIsValidPinUseCase = MockCheckIsValidPinUseCase();
    continuePinRecoveryUseCase = MockContinuePinRecoveryUseCase();
    cancelPinRecoveryUseCase = MockCancelPinRecoveryUseCase();
    completePinRecoveryUseCase = MockCompletePinRecoveryUseCase();
  });

  group('RecoverPinBloc', () {
    test('initial state is RecoverPinInitial when continueFromDigiD is false', () {
      final bloc = RecoverPinBloc(
        createPinRecoveryRedirectUriUseCase,
        checkIsValidPinUseCase,
        continuePinRecoveryUseCase,
        cancelPinRecoveryUseCase,
        completePinRecoveryUseCase,
        continueFromDigiD: false,
      );

      expect(bloc.state, const RecoverPinInitial());
    });

    test('initial state is RecoverPinVerifyingDigidAuthentication when continueFromDigiD is true', () {
      final bloc = RecoverPinBloc(
        createPinRecoveryRedirectUriUseCase,
        checkIsValidPinUseCase,
        continuePinRecoveryUseCase,
        cancelPinRecoveryUseCase,
        completePinRecoveryUseCase,
        continueFromDigiD: true,
      );

      expect(bloc.state, const RecoverPinVerifyingDigidAuthentication());
    });

    blocTest<RecoverPinBloc, RecoverPinState>(
      'happy path: login with DigiD, continue recovery, choose PIN, confirm PIN, succeed',
      build: () {
        when(createPinRecoveryRedirectUriUseCase.invoke())
            .thenAnswer((_) async => const Result.success('mock_auth_url'));
        when(continuePinRecoveryUseCase.invoke(any)).thenAnswer((_) async => const Result<void>.success(null));
        when(checkIsValidPinUseCase.invoke(any)).thenAnswer((_) async => const Result<void>.success(null));
        when(completePinRecoveryUseCase.invoke(any)).thenAnswer((_) async => const Result<void>.success(null));
        when(cancelPinRecoveryUseCase.invoke()).thenAnswer((_) async => const Result<void>.success(null));
        return RecoverPinBloc(
          createPinRecoveryRedirectUriUseCase,
          checkIsValidPinUseCase,
          continuePinRecoveryUseCase,
          cancelPinRecoveryUseCase,
          completePinRecoveryUseCase,
          continueFromDigiD: false,
        );
      },
      act: (bloc) async {
        bloc.add(const RecoverPinLoginWithDigidClicked());
        await Future.delayed(const Duration(milliseconds: 10));
        bloc.add(const RecoverPinContinuePinRecovery('mock_auth_url'));
        await Future.delayed(const Duration(milliseconds: 10));
        // Enter 6 digits for selecting a new PIN
        for (final d in [1, 2, 3, 4, 5, 5]) {
          bloc.add(RecoverPinDigitPressed(d));
        }
        await Future.delayed(const Duration(milliseconds: 10));
        // Enter 6 digits for confirming the selected PIN
        for (final d in [1, 2, 3, 4, 5, 5]) {
          bloc.add(RecoverPinDigitPressed(d));
        }
      },
      expect: () => [
        const RecoverPinLoadingDigidUrl(),
        const RecoverPinAwaitingDigidAuthentication('mock_auth_url'),
        const RecoverPinVerifyingDigidAuthentication(),
        const RecoverPinChooseNewPin(authUrl: 'mock_auth_url'),
        const RecoverPinChooseNewPin(authUrl: 'mock_auth_url', pin: '1'),
        const RecoverPinChooseNewPin(authUrl: 'mock_auth_url', pin: '12'),
        const RecoverPinChooseNewPin(authUrl: 'mock_auth_url', pin: '123'),
        const RecoverPinChooseNewPin(authUrl: 'mock_auth_url', pin: '1234'),
        const RecoverPinChooseNewPin(authUrl: 'mock_auth_url', pin: '12345'),
        const RecoverPinConfirmNewPin(authUrl: 'mock_auth_url', selectedPin: '123455', isRetrying: false),
        const RecoverPinConfirmNewPin(authUrl: 'mock_auth_url', selectedPin: '123455', isRetrying: false, pin: '1'),
        const RecoverPinConfirmNewPin(authUrl: 'mock_auth_url', selectedPin: '123455', isRetrying: false, pin: '12'),
        const RecoverPinConfirmNewPin(authUrl: 'mock_auth_url', selectedPin: '123455', isRetrying: false, pin: '123'),
        const RecoverPinConfirmNewPin(authUrl: 'mock_auth_url', selectedPin: '123455', isRetrying: false, pin: '1234'),
        const RecoverPinConfirmNewPin(authUrl: 'mock_auth_url', selectedPin: '123455', isRetrying: false, pin: '12345'),
        const RecoverPinUpdatingPin(),
        const RecoverPinSuccess(),
      ],
    );

    blocTest<RecoverPinBloc, RecoverPinState>(
      'handles back navigation from RecoverPinConfirmNewPin to RecoverPinChooseNewPin with didGoBack',
      build: () => RecoverPinBloc(
        createPinRecoveryRedirectUriUseCase,
        checkIsValidPinUseCase,
        continuePinRecoveryUseCase,
        cancelPinRecoveryUseCase,
        completePinRecoveryUseCase,
        continueFromDigiD: false,
      ),
      seed: () => const RecoverPinConfirmNewPin(
        authUrl: 'auth',
        selectedPin: '123456',
        isRetrying: false,
      ),
      act: (bloc) => bloc.add(const RecoverPinBackPressed()),
      expect: () => [const RecoverPinChooseNewPin(authUrl: 'auth', didGoBack: true)],
    );

    blocTest<RecoverPinBloc, RecoverPinState>(
      'handles back navigation from RecoverPinChooseNewPin to RecoverPinInitial with didGoBack',
      build: () => RecoverPinBloc(
        createPinRecoveryRedirectUriUseCase,
        checkIsValidPinUseCase,
        continuePinRecoveryUseCase,
        cancelPinRecoveryUseCase,
        completePinRecoveryUseCase,
        continueFromDigiD: false,
      ),
      seed: () => const RecoverPinChooseNewPin(authUrl: 'auth', pin: '12'),
      act: (bloc) => bloc.add(const RecoverPinBackPressed()),
      expect: () => [const RecoverPinInitial(didGoBack: true)],
    );

    blocTest<RecoverPinBloc, RecoverPinState>(
      'handles network error when getting DigiD URL',
      build: () {
        when(createPinRecoveryRedirectUriUseCase.invoke())
            .thenAnswer((_) async => const Result<String>.error(NetworkError(hasInternet: false, sourceError: 'test')));
        when(cancelPinRecoveryUseCase.invoke()).thenAnswer((_) async => const Result<void>.success(null));
        return RecoverPinBloc(
          createPinRecoveryRedirectUriUseCase,
          checkIsValidPinUseCase,
          continuePinRecoveryUseCase,
          cancelPinRecoveryUseCase,
          completePinRecoveryUseCase,
          continueFromDigiD: false,
        );
      },
      act: (bloc) => bloc.add(const RecoverPinLoginWithDigidClicked()),
      expect: () => [
        const RecoverPinLoadingDigidUrl(),
        isA<RecoverPinNetworkError>(),
      ],
    );

    blocTest<RecoverPinBloc, RecoverPinState>(
      'handles generic error when getting DigiD URL',
      build: () {
        when(createPinRecoveryRedirectUriUseCase.invoke())
            .thenAnswer((_) async => const Result<String>.error(GenericError('err', sourceError: 'ex')));
        when(cancelPinRecoveryUseCase.invoke()).thenAnswer((_) async => const Result<void>.success(null));
        return RecoverPinBloc(
          createPinRecoveryRedirectUriUseCase,
          checkIsValidPinUseCase,
          continuePinRecoveryUseCase,
          cancelPinRecoveryUseCase,
          completePinRecoveryUseCase,
          continueFromDigiD: false,
        );
      },
      act: (bloc) => bloc.add(const RecoverPinLoginWithDigidClicked()),
      expect: () => [
        const RecoverPinLoadingDigidUrl(),
        isA<RecoverPinGenericError>(),
      ],
    );

    blocTest<RecoverPinBloc, RecoverPinState>(
      'emits RecoverPinDigidLoginCancelled when DigiD reports user cancellation through RedirectUriError',
      build: () => RecoverPinBloc(
        createPinRecoveryRedirectUriUseCase,
        checkIsValidPinUseCase,
        continuePinRecoveryUseCase,
        cancelPinRecoveryUseCase,
        completePinRecoveryUseCase,
        continueFromDigiD: false,
      ),
      act: (bloc) => bloc.add(
        const RecoverPinLoginWithDigidFailed(
          error: RedirectUriError(redirectError: RedirectError.loginRequired, sourceError: 'mock'),
        ),
      ),
      expect: () => [isA<RecoverPinDigidLoginCancelled>()],
    );

    blocTest<RecoverPinBloc, RecoverPinState>(
      'emits RecoverPinDigidLoginCancelled when cancelledByUser = true',
      build: () => RecoverPinBloc(
        createPinRecoveryRedirectUriUseCase,
        checkIsValidPinUseCase,
        continuePinRecoveryUseCase,
        cancelPinRecoveryUseCase,
        completePinRecoveryUseCase,
        continueFromDigiD: false,
      ),
      act: (bloc) => bloc.add(
        const RecoverPinLoginWithDigidFailed(
          cancelledByUser: true,
          error: GenericError('cancelled', sourceError: 'test'),
        ),
      ),
      expect: () => [isA<RecoverPinDigidLoginCancelled>()],
    );

    blocTest<RecoverPinBloc, RecoverPinState>(
      'emits RecoverPinStopped on stop',
      build: () {
        when(cancelPinRecoveryUseCase.invoke()).thenAnswer((_) async => const Result<void>.success(null));
        return RecoverPinBloc(
          createPinRecoveryRedirectUriUseCase,
          checkIsValidPinUseCase,
          continuePinRecoveryUseCase,
          cancelPinRecoveryUseCase,
          completePinRecoveryUseCase,
          continueFromDigiD: false,
        );
      },
      act: (bloc) => bloc.add(const RecoverPinStopPressed()),
      expect: () => [const RecoverPinStopped()],
    );

    blocTest<RecoverPinBloc, RecoverPinState>(
      'retry: RecoverPinRetryPressed triggers RecoverPinLoginWithDigidClicked flow',
      build: () {
        when(createPinRecoveryRedirectUriUseCase.invoke())
            .thenAnswer((_) async => const Result.success('mock_auth_url'));
        when(cancelPinRecoveryUseCase.invoke()).thenAnswer((_) async => const Result<void>.success(null));
        return RecoverPinBloc(
          createPinRecoveryRedirectUriUseCase,
          checkIsValidPinUseCase,
          continuePinRecoveryUseCase,
          cancelPinRecoveryUseCase,
          completePinRecoveryUseCase,
          continueFromDigiD: false,
        );
      },
      act: (bloc) => bloc.add(const RecoverPinRetryPressed()),
      expect: () => [
        const RecoverPinLoadingDigidUrl(),
        const RecoverPinAwaitingDigidAuthentication('mock_auth_url'),
      ],
    );

    blocTest<RecoverPinBloc, RecoverPinState>(
      'pin validation failure on selecting new PIN emits RecoverPinSelectPinFailed then resets to ChooseNewPin',
      build: () {
        when(checkIsValidPinUseCase.invoke(any)).thenAnswer(
          (_) async => const Result<void>.error(
            ValidatePinError(PinValidationError.sequentialDigits, sourceError: 'x'),
          ),
        );
        return RecoverPinBloc(
          createPinRecoveryRedirectUriUseCase,
          checkIsValidPinUseCase,
          continuePinRecoveryUseCase,
          cancelPinRecoveryUseCase,
          completePinRecoveryUseCase,
          continueFromDigiD: false,
        );
      },
      seed: () => const RecoverPinChooseNewPin(authUrl: 'auth'),
      act: (bloc) async {
        for (final d in [1, 2, 3, 4, 5, 6]) {
          bloc.add(RecoverPinDigitPressed(d));
        }
      },
      expect: () => [
        const RecoverPinChooseNewPin(authUrl: 'auth', pin: '1'),
        const RecoverPinChooseNewPin(authUrl: 'auth', pin: '12'),
        const RecoverPinChooseNewPin(authUrl: 'auth', pin: '123'),
        const RecoverPinChooseNewPin(authUrl: 'auth', pin: '1234'),
        const RecoverPinChooseNewPin(authUrl: 'auth', pin: '12345'),
        isA<RecoverPinSelectPinFailed>(),
        const RecoverPinChooseNewPin(authUrl: 'auth'),
      ],
    );

    blocTest<RecoverPinBloc, RecoverPinState>(
      'confirm PIN mismatch (first attempt): emits failure with canRetry true and resets pin with isRetrying true',
      build: () => RecoverPinBloc(
        createPinRecoveryRedirectUriUseCase,
        checkIsValidPinUseCase,
        continuePinRecoveryUseCase,
        cancelPinRecoveryUseCase,
        completePinRecoveryUseCase,
        continueFromDigiD: false,
      ),
      seed: () => const RecoverPinConfirmNewPin(authUrl: 'auth', selectedPin: '123455', isRetrying: false),
      act: (bloc) async {
        for (final d in [0, 0, 0, 0, 0, 0]) {
          bloc.add(RecoverPinDigitPressed(d));
        }
      },
      expect: () => [
        const RecoverPinConfirmNewPin(authUrl: 'auth', selectedPin: '123455', isRetrying: false, pin: '0'),
        const RecoverPinConfirmNewPin(authUrl: 'auth', selectedPin: '123455', isRetrying: false, pin: '00'),
        const RecoverPinConfirmNewPin(authUrl: 'auth', selectedPin: '123455', isRetrying: false, pin: '000'),
        const RecoverPinConfirmNewPin(authUrl: 'auth', selectedPin: '123455', isRetrying: false, pin: '0000'),
        const RecoverPinConfirmNewPin(authUrl: 'auth', selectedPin: '123455', isRetrying: false, pin: '00000'),
        isA<RecoverPinConfirmPinFailed>().having((e) => e.canRetry, 'canRetry should be true', true),
        const RecoverPinConfirmNewPin(authUrl: 'auth', selectedPin: '123455', isRetrying: true, pin: ''),
      ],
    );

    blocTest<RecoverPinBloc, RecoverPinState>(
      'confirm PIN mismatch (second attempt): emits failure with canRetry false then navigates back to ChooseNewPin',
      build: () => RecoverPinBloc(
        createPinRecoveryRedirectUriUseCase,
        checkIsValidPinUseCase,
        continuePinRecoveryUseCase,
        cancelPinRecoveryUseCase,
        completePinRecoveryUseCase,
        continueFromDigiD: false,
      ),
      seed: () => const RecoverPinConfirmNewPin(authUrl: 'auth', selectedPin: '123456', isRetrying: true),
      act: (bloc) async {
        for (final d in [1, 1, 1, 1, 1, 1]) {
          bloc.add(RecoverPinDigitPressed(d));
        }
      },
      expect: () => [
        const RecoverPinConfirmNewPin(authUrl: 'auth', selectedPin: '123456', isRetrying: true, pin: '1'),
        const RecoverPinConfirmNewPin(authUrl: 'auth', selectedPin: '123456', isRetrying: true, pin: '11'),
        const RecoverPinConfirmNewPin(authUrl: 'auth', selectedPin: '123456', isRetrying: true, pin: '111'),
        const RecoverPinConfirmNewPin(authUrl: 'auth', selectedPin: '123456', isRetrying: true, pin: '1111'),
        const RecoverPinConfirmNewPin(authUrl: 'auth', selectedPin: '123456', isRetrying: true, pin: '11111'),
        isA<RecoverPinConfirmPinFailed>().having((e) => e.canRetry, 'canRetry should be false', false),
        const RecoverPinChooseNewPin(authUrl: 'auth'),
      ],
    );

    blocTest<RecoverPinBloc, RecoverPinState>(
      'backspace updates pin correctly in ChooseNewPin and ConfirmNewPin',
      build: () => RecoverPinBloc(
        createPinRecoveryRedirectUriUseCase,
        checkIsValidPinUseCase,
        continuePinRecoveryUseCase,
        cancelPinRecoveryUseCase,
        completePinRecoveryUseCase,
        continueFromDigiD: false,
      ),
      seed: () => const RecoverPinChooseNewPin(authUrl: 'auth', pin: '123'),
      act: (bloc) => bloc.add(RecoverPinBackspacePressed()),
      expect: () => [const RecoverPinChooseNewPin(authUrl: 'auth', pin: '12')],
    );

    blocTest<RecoverPinBloc, RecoverPinState>(
      'backspace updates pin correctly in ConfirmNewPin',
      build: () => RecoverPinBloc(
        createPinRecoveryRedirectUriUseCase,
        checkIsValidPinUseCase,
        continuePinRecoveryUseCase,
        cancelPinRecoveryUseCase,
        completePinRecoveryUseCase,
        continueFromDigiD: false,
      ),
      seed: () => const RecoverPinConfirmNewPin(authUrl: 'auth', selectedPin: '123456', pin: '789', isRetrying: false),
      act: (bloc) => bloc.add(RecoverPinBackspacePressed()),
      expect: () => [
        const RecoverPinConfirmNewPin(authUrl: 'auth', selectedPin: '123456', pin: '78', isRetrying: false),
      ],
    );

    blocTest<RecoverPinBloc, RecoverPinState>(
      'clear resets pin in ChooseNewPin and ConfirmNewPin',
      build: () => RecoverPinBloc(
        createPinRecoveryRedirectUriUseCase,
        checkIsValidPinUseCase,
        continuePinRecoveryUseCase,
        cancelPinRecoveryUseCase,
        completePinRecoveryUseCase,
        continueFromDigiD: false,
      ),
      seed: () => const RecoverPinChooseNewPin(authUrl: 'auth', pin: '123'),
      act: (bloc) => bloc.add(RecoverPinClearPressed()),
      expect: () => [const RecoverPinChooseNewPin(authUrl: 'auth', pin: '')],
    );

    blocTest<RecoverPinBloc, RecoverPinState>(
      'clear resets pin in ConfirmNewPin',
      build: () => RecoverPinBloc(
        createPinRecoveryRedirectUriUseCase,
        checkIsValidPinUseCase,
        continuePinRecoveryUseCase,
        cancelPinRecoveryUseCase,
        completePinRecoveryUseCase,
        continueFromDigiD: false,
      ),
      seed: () => const RecoverPinConfirmNewPin(authUrl: 'auth', selectedPin: '123456', pin: '123', isRetrying: false),
      act: (bloc) => bloc.add(RecoverPinClearPressed()),
      expect: () => [
        const RecoverPinConfirmNewPin(authUrl: 'auth', selectedPin: '123456', pin: '', isRetrying: false),
      ],
    );
  });
}
