import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/history/detail/bloc/history_detail_bloc.dart';

import '../../../../mocks/wallet_mock_data.dart';

void main() {
  blocTest(
    'verify initial state',
    build: HistoryDetailBloc.new,
    verify: (bloc) => expect(bloc.state, const HistoryDetailInitial()),
  );

  blocTest(
    'verify transition to HistoryDetailLoadSuccess when cards can be loaded',
    build: HistoryDetailBloc.new,
    act: (bloc) => bloc.add(HistoryDetailLoadTriggered(event: WalletMockData.disclosureEvent)),
    expect: () => [HistoryDetailLoadSuccess(WalletMockData.disclosureEvent)],
  );
}
