import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/model/wallet_card.dart';
import 'package:wallet/src/feature/card/history/bloc/card_history_bloc.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockGetWalletCardUseCase getWalletCardUseCase;
  late MockGetWalletEventsForCardUseCase getWalletEventsForCardUseCase;

  setUp(() {
    getWalletCardUseCase = MockGetWalletCardUseCase();
    getWalletEventsForCardUseCase = MockGetWalletEventsForCardUseCase();
    provideDummy<Result<WalletCard>>(Result.success(WalletMockData.card));
    provideDummy<Result<List<WalletEvent>>>(const Result.success([]));
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
          .thenAnswer((_) async => Result.success(WalletMockData.card));
      when(getWalletEventsForCardUseCase.invoke(WalletMockData.card.docType))
          .thenAnswer((_) => Future.value(const Result.success([])));
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
          .thenAnswer((_) async => const Result.error(GenericError('failed to load card', sourceError: 'test')));
    },
    act: (bloc) => bloc.add(CardHistoryLoadTriggered(WalletMockData.card.docType)),
    expect: () => [const CardHistoryLoadInProgress(), const CardHistoryLoadFailure()],
  );
}
