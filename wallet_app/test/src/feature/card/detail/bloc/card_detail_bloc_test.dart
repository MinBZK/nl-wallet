import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/feature/card/detail/bloc/card_detail_bloc.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockObserveWalletCardDetailUseCase observeWalletCardDetailUseCase;

  setUp(() {
    observeWalletCardDetailUseCase = MockObserveWalletCardDetailUseCase();
  });

  blocTest(
    'verify initial state',
    build: () => CardDetailBloc(observeWalletCardDetailUseCase, null),
    verify: (bloc) => expect(bloc.state, CardDetailInitial()),
  );

  blocTest(
    'verify loading state with preloaded card',
    build: () => CardDetailBloc(observeWalletCardDetailUseCase, WalletMockData.card),
    verify: (bloc) => expect(bloc.state, CardDetailLoadInProgress(card: WalletMockData.card)),
  );

  blocTest(
    'verify loading state without preloaded card',
    build: () => CardDetailBloc(observeWalletCardDetailUseCase, null),
    act: (bloc) => bloc.add(CardDetailLoadTriggered(WalletMockData.card.id!)),
    expect: () => [const CardDetailLoadInProgress()],
  );

  blocTest(
    'verify loading state with mismatched preloaded card',
    build: () => CardDetailBloc(observeWalletCardDetailUseCase, WalletMockData.altCard),
    act: (bloc) => bloc.add(CardDetailLoadTriggered(WalletMockData.card.id!)),
    expect: () => [const CardDetailLoadInProgress()],
  );

  blocTest(
    'verify success state with preloaded card',
    build: () => CardDetailBloc(observeWalletCardDetailUseCase, WalletMockData.card),
    act: (bloc) => bloc.add(CardDetailLoadTriggered(WalletMockData.card.id!)),
    setUp: () {
      when(observeWalletCardDetailUseCase.invoke(WalletMockData.card.id))
          .thenAnswer((_) => Stream.value(WalletMockData.cardDetail));
    },
    expect: () => [CardDetailLoadSuccess(WalletMockData.cardDetail)],
  );

  blocTest(
    'verify error state with preloaded card',
    build: () => CardDetailBloc(observeWalletCardDetailUseCase, WalletMockData.card),
    act: (bloc) => bloc.add(CardDetailLoadTriggered(WalletMockData.card.id!)),
    setUp: () {
      when(observeWalletCardDetailUseCase.invoke(WalletMockData.card.id))
          .thenAnswer((_) => Stream.error('Failed to load card details'));
    },
    expect: () => [CardDetailLoadFailure(WalletMockData.card.id!)],
  );
}
