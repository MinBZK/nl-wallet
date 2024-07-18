import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/feature/history/overview/bloc/history_overview_bloc.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.dart';

void main() {
  late MockGetWalletEventsUseCase getWalletEventsUseCase;

  setUp(() {
    getWalletEventsUseCase = MockGetWalletEventsUseCase();
  });

  blocTest(
    'verify initial state',
    build: () => HistoryOverviewBloc(getWalletEventsUseCase),
    verify: (bloc) => expect(bloc.state, HistoryOverviewInitial()),
  );

  blocTest(
    'verify transition to HistoryOverviewLoadFailure when events can not be loaded',
    build: () => HistoryOverviewBloc(getWalletEventsUseCase),
    setUp: () => when(getWalletEventsUseCase.invoke()).thenAnswer((_) => Future.error('Could not load cards')),
    act: (bloc) => bloc.add(const HistoryOverviewLoadTriggered()),
    expect: () => [const HistoryOverviewLoadInProgress(), const HistoryOverviewLoadFailure()],
  );

  blocTest(
    'verify transition to HistoryOverviewLoadSuccess when events can be loaded',
    build: () => HistoryOverviewBloc(getWalletEventsUseCase),
    setUp: () => when(getWalletEventsUseCase.invoke()).thenAnswer((_) async => [WalletMockData.disclosureEvent]),
    act: (bloc) => bloc.add(const HistoryOverviewLoadTriggered()),
    expect: () => [
      const HistoryOverviewLoadInProgress(),
      HistoryOverviewLoadSuccess([WalletMockData.disclosureEvent]),
    ],
  );
}
