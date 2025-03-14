import 'package:bloc_test/bloc_test.dart';
import 'package:mockito/mockito.dart';
import 'package:test/test.dart';
import 'package:wallet/src/domain/usecase/card/observe_wallet_cards_usecase.dart';
import 'package:wallet/src/domain/usecase/event/observe_recent_wallet_events_usecase.dart';
import 'package:wallet/src/feature/dashboard/bloc/dashboard_bloc.dart';

import '../../../mocks/wallet_mock_data.dart';
import '../../../mocks/wallet_mocks.dart';

void main() {
  late ObserveWalletCardsUseCase observeWalletCardsUseCase;
  late ObserveRecentWalletEventsUseCase observeRecentWalletEventUseCase;

  setUp(() {
    observeWalletCardsUseCase = MockObserveWalletCardsUseCase();
    observeRecentWalletEventUseCase = MockObserveRecentWalletEventsUseCase();
  });

  blocTest(
    'verify initial state without preloaded cards',
    build: () => DashboardBloc(
      observeWalletCardsUseCase,
      observeRecentWalletEventUseCase,
      null,
    ),
    verify: (bloc) {
      expect(bloc.state, const DashboardStateInitial());
    },
  );

  blocTest(
    'verify initial state with preloaded cards',
    build: () => DashboardBloc(
      observeWalletCardsUseCase,
      observeRecentWalletEventUseCase,
      [WalletMockData.card],
    ),
    verify: (bloc) {
      expect(bloc.state, DashboardLoadSuccess(cards: [WalletMockData.card]));
    },
  );

  blocTest(
    'verify loading state',
    build: () => DashboardBloc(
      observeWalletCardsUseCase,
      observeRecentWalletEventUseCase,
      null,
    ),
    act: (bloc) => bloc.add(const DashboardLoadTriggered()),
    expect: () => [const DashboardLoadInProgress()],
  );

  blocTest(
    'verify no loading state when state is success',
    build: () => DashboardBloc(
      observeWalletCardsUseCase,
      observeRecentWalletEventUseCase,
      [WalletMockData.card],
    ),
    act: (bloc) => bloc.add(const DashboardLoadTriggered()),
    expect: () => [],
  );

  blocTest(
    'verify loading state when state is success but refresh is forced',
    build: () => DashboardBloc(
      observeWalletCardsUseCase,
      observeRecentWalletEventUseCase,
      [WalletMockData.card],
    ),
    act: (bloc) => bloc.add(const DashboardLoadTriggered(forceRefresh: true)),
    expect: () => [const DashboardLoadInProgress()],
  );

  blocTest(
    'verify cards and history are fetched through usecases',
    build: () => DashboardBloc(
      observeWalletCardsUseCase,
      observeRecentWalletEventUseCase,
      null,
    ),
    setUp: () {
      when(observeWalletCardsUseCase.invoke()).thenAnswer((_) => Stream.value([WalletMockData.altCard]));
      when(observeRecentWalletEventUseCase.invoke()).thenAnswer((_) => Stream.value([WalletMockData.disclosureEvent]));
    },
    act: (bloc) => bloc.add(const DashboardLoadTriggered()),
    expect: () => [
      const DashboardLoadInProgress(),
      DashboardLoadSuccess(
        cards: [WalletMockData.altCard],
        history: [WalletMockData.disclosureEvent],
      ),
    ],
  );

  blocTest(
    'verify failure is emitted when history cant be loaded',
    build: () => DashboardBloc(
      observeWalletCardsUseCase,
      observeRecentWalletEventUseCase,
      null,
    ),
    setUp: () =>
        when(observeRecentWalletEventUseCase.invoke()).thenAnswer((_) => Stream.error('failed to load history')),
    act: (bloc) => bloc.add(const DashboardLoadTriggered()),
    expect: () => [
      const DashboardLoadInProgress(),
      const DashboardLoadFailure(),
    ],
  );

  blocTest(
    'verify failure is emitted when cards cant be loaded',
    build: () => DashboardBloc(
      observeWalletCardsUseCase,
      observeRecentWalletEventUseCase,
      null,
    ),
    setUp: () {
      when(observeWalletCardsUseCase.invoke()).thenAnswer((_) => Stream.error('Failed to load cards'));
      when(observeRecentWalletEventUseCase.invoke()).thenAnswer((_) => Stream.value([WalletMockData.disclosureEvent]));
    },
    act: (bloc) => bloc.add(const DashboardLoadTriggered()),
    expect: () => [
      const DashboardLoadInProgress(),
      const DashboardLoadFailure(),
    ],
  );
}
