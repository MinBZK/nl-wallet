import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/flow_progress.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/model/wallet_card.dart';
import 'package:wallet/src/feature/wallet/personalize/bloc/wallet_personalize_bloc.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.dart';

void main() {
  late MockGetWalletCardsUseCase mockGetWalletCardsUseCase;
  late MockGetPidIssuanceUrlUseCase mockGetPidIssuanceUrlUseCase;
  late MockCancelPidIssuanceUseCase mockCancelPidIssuanceUseCase;
  late MockContinuePidIssuanceUseCase mockContinuePidIssuanceUseCase;
  late MockIsWalletInitializedWithPidUseCase mockIsWalletInitializedWithPidUseCase;

  setUp(() async {
    mockGetWalletCardsUseCase = MockGetWalletCardsUseCase();
    mockGetPidIssuanceUrlUseCase = MockGetPidIssuanceUrlUseCase();
    mockCancelPidIssuanceUseCase = MockCancelPidIssuanceUseCase();
    mockContinuePidIssuanceUseCase = MockContinuePidIssuanceUseCase();
    mockIsWalletInitializedWithPidUseCase = MockIsWalletInitializedWithPidUseCase();
    provideDummy<Result<List<WalletCard>>>(const Result.success([]));
    provideDummy<Result<String>>(const Result.success(''));
    provideDummy<Result<bool>>(const Result.success(true));
    provideDummy<Result<List<Attribute>>>(const Result.success([]));
  });

  blocTest(
    'verify initial state',
    build: () => WalletPersonalizeBloc(
      mockGetWalletCardsUseCase,
      mockGetPidIssuanceUrlUseCase,
      mockCancelPidIssuanceUseCase,
      mockContinuePidIssuanceUseCase,
      mockIsWalletInitializedWithPidUseCase,
    ),
    verify: (bloc) => expect(bloc.state, const WalletPersonalizeInitial()),
  );

  test('Verify app crashes when isInitializedWithPid throws a StateError', () {
    // This usecase should never throw, if it does we are in an invalid state and should just crash.
    when(mockIsWalletInitializedWithPidUseCase.invoke()).thenThrow(StateError('error'));

    expect(
      () => WalletPersonalizeBloc(
        mockGetWalletCardsUseCase,
        mockGetPidIssuanceUrlUseCase,
        mockCancelPidIssuanceUseCase,
        mockContinuePidIssuanceUseCase,
        mockIsWalletInitializedWithPidUseCase,
      ),
      throwsA(isA<StateError>()),
    );
  });

  blocTest(
    'verify initial state when wallet is initialized with pid',
    build: () => WalletPersonalizeBloc(
      mockGetWalletCardsUseCase,
      mockGetPidIssuanceUrlUseCase,
      mockCancelPidIssuanceUseCase,
      mockContinuePidIssuanceUseCase,
      mockIsWalletInitializedWithPidUseCase,
    ),
    setUp: () {
      when(mockGetWalletCardsUseCase.invoke()).thenAnswer((_) async => Result.success([WalletMockData.card]));
      when(mockIsWalletInitializedWithPidUseCase.invoke()).thenAnswer((_) => Future.value(true));
    },
    wait: const Duration(milliseconds: 50),
    verify: (bloc) {
      expect(bloc.state, WalletPersonalizeSuccess([WalletMockData.card]));
    },
  );

  blocTest(
    'verify successful path to pid issuance',
    build: () => WalletPersonalizeBloc(
      mockGetWalletCardsUseCase,
      mockGetPidIssuanceUrlUseCase,
      mockCancelPidIssuanceUseCase,
      mockContinuePidIssuanceUseCase,
      mockIsWalletInitializedWithPidUseCase,
    ),
    setUp: () {
      when(mockGetWalletCardsUseCase.invoke()).thenAnswer((_) async => Result.success([WalletMockData.card]));
      when(mockGetPidIssuanceUrlUseCase.invoke()).thenAnswer((_) async => const Result.success('pid_issuance_url'));
    },
    act: (bloc) async {
      bloc.add(WalletPersonalizeLoginWithDigidClicked());
      await Future.delayed(const Duration(milliseconds: 10));
      bloc.add(WalletPersonalizeLoginWithDigidSucceeded(WalletMockData.card.attributes));
      await Future.delayed(const Duration(milliseconds: 10));
      bloc.add(WalletPersonalizeOfferingAccepted(WalletMockData.card.attributes));
      await Future.delayed(const Duration(milliseconds: 10));
      bloc.add(WalletPersonalizePinConfirmed());
    },
    expect: () => [
      const WalletPersonalizeLoadingIssuanceUrl(),
      const WalletPersonalizeConnectDigid('pid_issuance_url'),
      WalletPersonalizeCheckData(availableAttributes: WalletMockData.card.attributes),
      WalletPersonalizeConfirmPin(WalletMockData.card.attributes),
      const WalletPersonalizeAddingCards(FlowProgress(currentStep: 8, totalSteps: 9)),
      WalletPersonalizeSuccess([WalletMockData.card]),
    ],
  );

  blocTest(
    'verify getting issuance url error path',
    build: () => WalletPersonalizeBloc(
      mockGetWalletCardsUseCase,
      mockGetPidIssuanceUrlUseCase,
      mockCancelPidIssuanceUseCase,
      mockContinuePidIssuanceUseCase,
      mockIsWalletInitializedWithPidUseCase,
    ),
    setUp: () {
      when(mockGetWalletCardsUseCase.invoke()).thenAnswer((_) async => Result.success([WalletMockData.card]));
      when(mockGetPidIssuanceUrlUseCase.invoke()).thenAnswer(
        (_) async => const Result.error(GenericError('failed to get issuance url', sourceError: 'test')),
      );
    },
    act: (bloc) async {
      bloc.add(WalletPersonalizeLoginWithDigidClicked());
    },
    expect: () => [
      const WalletPersonalizeLoadingIssuanceUrl(),
      isA<WalletPersonalizeDigidFailure>(),
    ],
  );

  blocTest(
    'verify successful path from continuePidIssuance (i.e. deeplink back into app)',
    build: () => WalletPersonalizeBloc(
      mockGetWalletCardsUseCase,
      mockGetPidIssuanceUrlUseCase,
      mockCancelPidIssuanceUseCase,
      mockContinuePidIssuanceUseCase,
      mockIsWalletInitializedWithPidUseCase,
    ),
    setUp: () {
      when(mockGetWalletCardsUseCase.invoke()).thenAnswer((_) async => Result.success([WalletMockData.card]));
      when(mockGetPidIssuanceUrlUseCase.invoke()).thenAnswer((_) async => const Result.success('pid_issuance_url'));
      when(mockContinuePidIssuanceUseCase.invoke('auth_url')).thenAnswer((_) async => const Result.success([]));
    },
    act: (bloc) async {
      bloc.add(const WalletPersonalizeContinuePidIssuance('auth_url'));
      await Future.delayed(const Duration(milliseconds: 10));
      bloc.add(WalletPersonalizeOfferingAccepted(WalletMockData.card.attributes));
      await Future.delayed(const Duration(milliseconds: 10));
      bloc.add(WalletPersonalizePinConfirmed());
    },
    expect: () => [
      const WalletPersonalizeAuthenticating(),
      const WalletPersonalizeCheckData(availableAttributes: []),
      WalletPersonalizeConfirmPin(WalletMockData.card.attributes),
      const WalletPersonalizeAddingCards(FlowProgress(currentStep: 8, totalSteps: 9)),
      WalletPersonalizeSuccess([WalletMockData.card]),
    ],
  );

  blocTest(
    'verify that accessDenied error leads to WalletPersonalizeDigidCancelled',
    build: () => WalletPersonalizeBloc(
      mockGetWalletCardsUseCase,
      mockGetPidIssuanceUrlUseCase,
      mockCancelPidIssuanceUseCase,
      mockContinuePidIssuanceUseCase,
      mockIsWalletInitializedWithPidUseCase,
    ),
    setUp: () {
      when(mockGetWalletCardsUseCase.invoke()).thenAnswer((_) async => Result.success([WalletMockData.card]));
      when(mockGetPidIssuanceUrlUseCase.invoke()).thenAnswer((_) async => const Result.success('pid_issuance_url'));
      when(mockContinuePidIssuanceUseCase.invoke('auth_url')).thenAnswer(
        (_) async => const Result.error(RedirectUriError(redirectError: RedirectError.accessDenied, sourceError: '')),
      );
    },
    act: (bloc) async {
      bloc.add(const WalletPersonalizeContinuePidIssuance('auth_url'));
    },
    expect: () => [
      const WalletPersonalizeAuthenticating(),
      isA<WalletPersonalizeLoadInProgress>(),
      WalletPersonalizeDigidCancelled(),
    ],
  );

  blocTest(
    'rejecting the offered pid puts the user back at the initial state',
    build: () => WalletPersonalizeBloc(
      mockGetWalletCardsUseCase,
      mockGetPidIssuanceUrlUseCase,
      mockCancelPidIssuanceUseCase,
      mockContinuePidIssuanceUseCase,
      mockIsWalletInitializedWithPidUseCase,
    ),
    setUp: () {
      when(mockContinuePidIssuanceUseCase.invoke('auth_url')).thenAnswer((_) async => const Result.success([]));
    },
    act: (bloc) async {
      bloc.add(const WalletPersonalizeContinuePidIssuance('auth_url'));
      await Future.delayed(const Duration(milliseconds: 10));
      bloc.add(WalletPersonalizeOfferingRejected());
    },
    expect: () => [
      const WalletPersonalizeAuthenticating(),
      const WalletPersonalizeCheckData(availableAttributes: []),
      isA<WalletPersonalizeLoadInProgress>(),
      const WalletPersonalizeInitial(),
    ],
  );

  blocTest(
    'when accepting the pid fails with a generic error, the bloc transitions to WalletPersonalizeGenericError',
    build: () => WalletPersonalizeBloc(
      mockGetWalletCardsUseCase,
      mockGetPidIssuanceUrlUseCase,
      mockCancelPidIssuanceUseCase,
      mockContinuePidIssuanceUseCase,
      mockIsWalletInitializedWithPidUseCase,
    ),
    setUp: () {
      when(mockContinuePidIssuanceUseCase.invoke('auth_url')).thenAnswer(
        (_) async => Result.success(WalletMockData.card.attributes),
      );
    },
    act: (bloc) async {
      bloc.add(const WalletPersonalizeContinuePidIssuance('auth_url'));
      await Future.delayed(const Duration(milliseconds: 10));
      bloc.add(WalletPersonalizeOfferingAccepted(WalletMockData.card.attributes));
      await Future.delayed(const Duration(milliseconds: 10));
      bloc.add(const WalletPersonalizeAcceptPidFailed(error: GenericError('error', sourceError: 'test')));
    },
    expect: () => [
      const WalletPersonalizeAuthenticating(),
      WalletPersonalizeCheckData(availableAttributes: WalletMockData.card.attributes),
      WalletPersonalizeConfirmPin(WalletMockData.card.attributes),
      isA<WalletPersonalizeLoadInProgress>(),
      isA<WalletPersonalizeGenericError>(),
    ],
  );
}
