import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/flow_progress.dart';
import 'package:wallet/src/feature/wallet/personalize/bloc/wallet_personalize_bloc.dart';

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
    verify: (bloc) {
      expect(bloc.state, const WalletPersonalizeInitial());
    },
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
      const WalletPersonalizeLoadInProgress(FlowProgress(currentStep: 8, totalSteps: 9)),
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
}
