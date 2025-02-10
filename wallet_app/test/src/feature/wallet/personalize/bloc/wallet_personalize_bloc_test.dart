import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/flow_progress.dart';
import 'package:wallet/src/domain/model/pid/pid_issuance_status.dart';
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

  blocTest(
    'verify initial state when isInitializedWithPid throws',
    build: () => WalletPersonalizeBloc(
      mockGetWalletCardsUseCase,
      mockGetPidIssuanceUrlUseCase,
      mockCancelPidIssuanceUseCase,
      mockContinuePidIssuanceUseCase,
      mockIsWalletInitializedWithPidUseCase,
    ),
    setUp: () => when(mockIsWalletInitializedWithPidUseCase.invoke()).thenAnswer((_) async => throw 'error'),
    verify: (bloc) => expect(bloc.state, const WalletPersonalizeInitial()),
  );

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
      when(mockGetWalletCardsUseCase.invoke()).thenAnswer((_) async => [WalletMockData.card]);
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
      when(mockGetWalletCardsUseCase.invoke()).thenAnswer((_) async => [WalletMockData.card]);
      when(mockGetPidIssuanceUrlUseCase.invoke()).thenAnswer((_) async => 'pid_issuance_url');
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
      when(mockGetWalletCardsUseCase.invoke()).thenAnswer((_) async => [WalletMockData.card]);
      when(mockGetPidIssuanceUrlUseCase.invoke()).thenAnswer((_) async => throw 'issuance_url_error');
    },
    act: (bloc) async {
      bloc.add(WalletPersonalizeLoginWithDigidClicked());
    },
    expect: () => [
      const WalletPersonalizeLoadingIssuanceUrl(),
      const WalletPersonalizeDigidFailure(error: 'issuance_url_error'),
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
      when(mockGetWalletCardsUseCase.invoke()).thenAnswer((_) async => [WalletMockData.card]);
      when(mockGetPidIssuanceUrlUseCase.invoke()).thenAnswer((_) async => 'pid_issuance_url');
      when(mockContinuePidIssuanceUseCase.invoke('auth_url')).thenAnswer((_) async => PidIssuanceSuccess(const []));
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
      when(mockGetWalletCardsUseCase.invoke()).thenAnswer((_) async => [WalletMockData.card]);
      when(mockGetPidIssuanceUrlUseCase.invoke()).thenAnswer((_) async => 'pid_issuance_url');
      when(mockContinuePidIssuanceUseCase.invoke('auth_url'))
          .thenAnswer((_) async => PidIssuanceError(RedirectError.accessDenied));
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
      when(mockContinuePidIssuanceUseCase.invoke('auth_url')).thenAnswer((_) async => PidIssuanceSuccess(const []));
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
        (_) async => PidIssuanceSuccess(WalletMockData.card.attributes),
      );
    },
    act: (bloc) async {
      bloc.add(const WalletPersonalizeContinuePidIssuance('auth_url'));
      await Future.delayed(const Duration(milliseconds: 10));
      bloc.add(WalletPersonalizeOfferingAccepted(WalletMockData.card.attributes));
      await Future.delayed(const Duration(milliseconds: 10));
      bloc.add(const WalletPersonalizeAcceptPidFailed(error: CoreGenericError('error')));
    },
    expect: () => [
      const WalletPersonalizeAuthenticating(),
      WalletPersonalizeCheckData(availableAttributes: WalletMockData.card.attributes),
      WalletPersonalizeConfirmPin(WalletMockData.card.attributes),
      isA<WalletPersonalizeLoadInProgress>(),
      const WalletPersonalizeGenericError(error: CoreGenericError('error')),
    ],
  );
}
