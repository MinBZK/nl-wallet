import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/feature/card/history/bloc/card_history_bloc.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockGetWalletCardUseCase getWalletCardUseCase;
  late MockGetWalletEventsForCardUseCase getWalletEventsForCardUseCase;
  late MockGetWalletEventsForPidUseCase getWalletEventsForPidUseCase;
  late MockCheckIsPidUseCase checkIsPidUseCase;

  setUp(() {
    getWalletCardUseCase = MockGetWalletCardUseCase();
    getWalletEventsForCardUseCase = MockGetWalletEventsForCardUseCase();
    getWalletEventsForPidUseCase = MockGetWalletEventsForPidUseCase();
    checkIsPidUseCase = MockCheckIsPidUseCase();
  });

  blocTest(
    'verify initial state',
    build: () => CardHistoryBloc(
      getWalletCardUseCase,
      getWalletEventsForCardUseCase,
      getWalletEventsForPidUseCase,
      checkIsPidUseCase,
    ),
    verify: (bloc) => expect(bloc.state, CardHistoryInitial()),
  );

  blocTest(
    'verify success state for non-PID card',
    build: () => CardHistoryBloc(
      getWalletCardUseCase,
      getWalletEventsForCardUseCase,
      getWalletEventsForPidUseCase,
      checkIsPidUseCase,
    ),
    setUp: () {
      when(
        getWalletCardUseCase.invoke(WalletMockData.card.attestationType),
      ).thenAnswer((_) async => Result.success(WalletMockData.card));
      when(
        checkIsPidUseCase.invoke(WalletMockData.card),
      ).thenAnswer((_) async => const Result.success(false));
      when(
        getWalletEventsForCardUseCase.invoke(WalletMockData.card.attestationType),
      ).thenAnswer((_) => Future.value(const Result.success([])));
    },
    act: (bloc) => bloc.add(CardHistoryLoadTriggered(WalletMockData.card.attestationType)),
    expect: () => [const CardHistoryLoadInProgress(), CardHistoryLoadSuccess(WalletMockData.card, const [])],
    verify: (_) {
      verify(getWalletEventsForCardUseCase.invoke(WalletMockData.card.attestationType)).called(1);
      verifyNever(getWalletEventsForPidUseCase.invoke());
    },
  );

  blocTest(
    'verify success state for PID card',
    build: () => CardHistoryBloc(
      getWalletCardUseCase,
      getWalletEventsForCardUseCase,
      getWalletEventsForPidUseCase,
      checkIsPidUseCase,
    ),
    setUp: () {
      when(
        getWalletCardUseCase.invoke(WalletMockData.card.attestationType),
      ).thenAnswer((_) async => Result.success(WalletMockData.card));
      when(
        checkIsPidUseCase.invoke(WalletMockData.card),
      ).thenAnswer((_) async => const Result.success(true));
      when(
        getWalletEventsForPidUseCase.invoke(),
      ).thenAnswer((_) => Future.value(const Result.success([])));
    },
    act: (bloc) => bloc.add(CardHistoryLoadTriggered(WalletMockData.card.attestationType)),
    expect: () => [const CardHistoryLoadInProgress(), CardHistoryLoadSuccess(WalletMockData.card, const [])],
    verify: (_) {
      verify(getWalletEventsForPidUseCase.invoke()).called(1);
      verifyNever(getWalletEventsForCardUseCase.invoke(any));
    },
  );

  blocTest(
    'verify error state',
    build: () => CardHistoryBloc(
      getWalletCardUseCase,
      getWalletEventsForCardUseCase,
      getWalletEventsForPidUseCase,
      checkIsPidUseCase,
    ),
    setUp: () {
      when(
        getWalletCardUseCase.invoke(WalletMockData.card.attestationType),
      ).thenAnswer((_) async => const Result.error(GenericError('failed to load card', sourceError: 'test')));
    },
    act: (bloc) => bloc.add(CardHistoryLoadTriggered(WalletMockData.card.attestationType)),
    expect: () => [const CardHistoryLoadInProgress(), const CardHistoryLoadFailure()],
  );
}
