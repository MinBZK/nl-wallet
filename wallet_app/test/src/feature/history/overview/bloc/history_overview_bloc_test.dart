import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/feature/history/overview/bloc/history_overview_bloc.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.dart';

void main() {
  late MockGetWalletEventsUseCase getWalletEventsUseCase;

  setUp(() {
    getWalletEventsUseCase = MockGetWalletEventsUseCase();
    provideDummy<Result<List<WalletEvent>>>(const Result.success([]));
  });

  blocTest(
    'verify initial state',
    build: () => HistoryOverviewBloc(getWalletEventsUseCase),
    verify: (bloc) => expect(bloc.state, HistoryOverviewInitial()),
  );

  blocTest(
    'verify transition to HistoryOverviewLoadFailure when events can not be loaded',
    build: () => HistoryOverviewBloc(getWalletEventsUseCase),
    setUp: () => when(getWalletEventsUseCase.invoke())
        .thenAnswer((_) async => const Result.error(GenericError('', sourceError: 'test'))),
    act: (bloc) => bloc.add(const HistoryOverviewLoadTriggered()),
    expect: () => [
      const HistoryOverviewLoadInProgress(),
      const HistoryOverviewLoadFailure(error: GenericError('', sourceError: 'test')),
    ],
  );

  blocTest(
    'verify transition to HistoryOverviewLoadSuccess when events can be loaded',
    build: () => HistoryOverviewBloc(getWalletEventsUseCase),
    setUp: () =>
        when(getWalletEventsUseCase.invoke()).thenAnswer((_) async => Result.success([WalletMockData.disclosureEvent])),
    act: (bloc) => bloc.add(const HistoryOverviewLoadTriggered()),
    expect: () => [
      const HistoryOverviewLoadInProgress(),
      HistoryOverviewLoadSuccess([WalletMockData.disclosureEvent]),
    ],
  );
}
