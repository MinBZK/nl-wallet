import 'package:bloc_test/bloc_test.dart';
import 'package:mockito/mockito.dart';
import 'package:test/test.dart';
import 'package:wallet/src/domain/usecase/card/observe_wallet_cards_usecase.dart';
import 'package:wallet/src/domain/usecase/history/get_wallet_timeline_attributes_usecase.dart';
import 'package:wallet/src/feature/dashboard/bloc/dashboard_bloc.dart';

import '../../../mocks/wallet_mock_data.dart';
import '../../../mocks/wallet_mocks.dart';

void main() {
  late ObserveWalletCardsUseCase observeWalletCardsUseCase;
  late GetWalletTimelineAttributesUseCase getWalletTimelineAttributesUseCase;

  setUp(() {
    observeWalletCardsUseCase = MockObserveWalletCardsUseCase();
    getWalletTimelineAttributesUseCase = MockGetWalletTimelineAttributesUseCase();
  });

  blocTest(
    'verify initial state without preloaded cards',
    build: () => DashboardBloc(
      observeWalletCardsUseCase,
      getWalletTimelineAttributesUseCase,
      null,
    ),
    verify: (bloc) => bloc.state == const DashboardStateInitial(),
  );

  blocTest(
    'verify initial state with preloaded cards',
    build: () => DashboardBloc(
      observeWalletCardsUseCase,
      getWalletTimelineAttributesUseCase,
      [WalletMockData.card],
    ),
    verify: (bloc) => bloc.state == DashboardLoadSuccess(cards: [WalletMockData.card]),
  );

  blocTest(
    'verify loading state',
    build: () => DashboardBloc(
      observeWalletCardsUseCase,
      getWalletTimelineAttributesUseCase,
      null,
    ),
    act: (bloc) => bloc.add(const DashboardLoadTriggered()),
    expect: () => [const DashboardLoadInProgress()],
  );

  blocTest(
    'verify no loading state when state is success',
    build: () => DashboardBloc(
      observeWalletCardsUseCase,
      getWalletTimelineAttributesUseCase,
      [WalletMockData.card],
    ),
    act: (bloc) => bloc.add(const DashboardLoadTriggered()),
    expect: () => [],
  );

  blocTest(
    'verify loading state when state is success but refresh is forced',
    build: () => DashboardBloc(
      observeWalletCardsUseCase,
      getWalletTimelineAttributesUseCase,
      [WalletMockData.card],
    ),
    act: (bloc) => bloc.add(const DashboardLoadTriggered(forceRefresh: true)),
    expect: () => [const DashboardLoadInProgress()],
  );

  blocTest(
    'verify cards and history are fetched through usecases',
    build: () => DashboardBloc(
      observeWalletCardsUseCase,
      getWalletTimelineAttributesUseCase,
      null,
    ),
    setUp: () {
      when(observeWalletCardsUseCase.invoke()).thenAnswer((_) => Stream.value([WalletMockData.altCard]));
      when(getWalletTimelineAttributesUseCase.invoke())
          .thenAnswer((_) async => [WalletMockData.interactionTimelineAttribute]);
    },
    act: (bloc) => bloc.add(const DashboardLoadTriggered()),
    expect: () => [
      const DashboardLoadInProgress(),
      DashboardLoadSuccess(
        cards: [WalletMockData.altCard],
        history: [WalletMockData.interactionTimelineAttribute],
      ),
    ],
  );

  blocTest(
    'verify failure is emitted when history cant be loaded',
    build: () => DashboardBloc(
      observeWalletCardsUseCase,
      getWalletTimelineAttributesUseCase,
      null,
    ),
    setUp: () => when(getWalletTimelineAttributesUseCase.invoke()).thenThrow('Failed to fetch history'),
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
      getWalletTimelineAttributesUseCase,
      null,
    ),
    setUp: () {
      when(observeWalletCardsUseCase.invoke()).thenAnswer((_) => Stream.error('Failed to load cards'));
      when(getWalletTimelineAttributesUseCase.invoke())
          .thenAnswer((_) async => [WalletMockData.interactionTimelineAttribute]);
    },
    act: (bloc) => bloc.add(const DashboardLoadTriggered()),
    expect: () => [
      const DashboardLoadInProgress(),
      const DashboardLoadFailure(),
    ],
  );
}
