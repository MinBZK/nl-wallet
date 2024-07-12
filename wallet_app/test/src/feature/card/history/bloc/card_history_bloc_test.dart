import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/feature/card/history/bloc/card_history_bloc.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockGetWalletCardUseCase getWalletCardUseCase;
  late MockGetWalletEventsForCardUseCase getWalletEventsForCardUseCase;

  setUp(() {
    getWalletCardUseCase = MockGetWalletCardUseCase();
    getWalletEventsForCardUseCase = MockGetWalletEventsForCardUseCase();
  });

  blocTest(
    'verify initial state',
    build: () => CardHistoryBloc(
      getWalletCardUseCase,
      getWalletEventsForCardUseCase,
    ),
    verify: (bloc) {
      expect(bloc.state, CardHistoryInitial());
    },
  );

  blocTest(
    'verify success state',
    build: () => CardHistoryBloc(
      getWalletCardUseCase,
      getWalletEventsForCardUseCase,
    ),
    setUp: () {
      when(getWalletCardUseCase.invoke(WalletMockData.card.docType))
          .thenAnswer((_) => Future.value(WalletMockData.card));
      when(getWalletEventsForCardUseCase.invoke(WalletMockData.card.docType)).thenAnswer((_) => Future.value([]));
    },
    act: (bloc) => bloc.add(CardHistoryLoadTriggered(WalletMockData.card.docType)),
    expect: () => [const CardHistoryLoadInProgress(), CardHistoryLoadSuccess(WalletMockData.card, const [])],
  );

  blocTest(
    'verify error state',
    build: () => CardHistoryBloc(
      getWalletCardUseCase,
      getWalletEventsForCardUseCase,
    ),
    setUp: () {
      when(getWalletCardUseCase.invoke(WalletMockData.card.docType))
          .thenAnswer((_) => Future.error('failed to load card data'));
    },
    act: (bloc) => bloc.add(CardHistoryLoadTriggered(WalletMockData.card.docType)),
    expect: () => [const CardHistoryLoadInProgress(), const CardHistoryLoadFailure()],
  );
}
