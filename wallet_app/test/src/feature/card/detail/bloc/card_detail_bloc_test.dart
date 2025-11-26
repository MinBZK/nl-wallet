import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/feature/card/detail/bloc/card_detail_bloc.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockObserveWalletCardDetailUseCase observeWalletCardDetailUseCase;
  late MockCheckIsPidUseCase checkIsPidUseCase;

  setUp(() {
    observeWalletCardDetailUseCase = MockObserveWalletCardDetailUseCase();
    checkIsPidUseCase = MockCheckIsPidUseCase();
  });

  blocTest(
    'verify initial state',
    build: () => CardDetailBloc(observeWalletCardDetailUseCase, checkIsPidUseCase, null),
    verify: (bloc) => expect(bloc.state, CardDetailInitial()),
  );

  blocTest(
    'verify loading state with preloaded card',
    build: () => CardDetailBloc(observeWalletCardDetailUseCase, checkIsPidUseCase, WalletMockData.card),
    verify: (bloc) => expect(bloc.state, CardDetailLoadInProgress(card: WalletMockData.card)),
  );

  blocTest(
    'verify loading state without preloaded card',
    build: () => CardDetailBloc(observeWalletCardDetailUseCase, checkIsPidUseCase, null),
    act: (bloc) => bloc.add(CardDetailLoadTriggered(WalletMockData.card.attestationId!)),
    expect: () => [const CardDetailLoadInProgress()],
  );

  blocTest(
    'verify loading state with mismatched preloaded card',
    build: () => CardDetailBloc(observeWalletCardDetailUseCase, checkIsPidUseCase, WalletMockData.altCard),
    act: (bloc) => bloc.add(CardDetailLoadTriggered(WalletMockData.card.attestationId!)),
    expect: () => [const CardDetailLoadInProgress()],
  );

  blocTest(
    'verify success state with preloaded card',
    build: () => CardDetailBloc(observeWalletCardDetailUseCase, checkIsPidUseCase, WalletMockData.card),
    act: (bloc) => bloc.add(CardDetailLoadTriggered(WalletMockData.card.attestationId!)),
    setUp: () {
      when(checkIsPidUseCase.invoke(any)).thenAnswer((_) async => const Result.success(false));
      when(
        observeWalletCardDetailUseCase.invoke(WalletMockData.card.attestationId),
      ).thenAnswer((_) => Stream.value(WalletMockData.cardDetail));
    },
    expect: () => [CardDetailLoadSuccess(WalletMockData.cardDetail)],
  );

  blocTest(
    'verify error state with preloaded card',
    build: () => CardDetailBloc(observeWalletCardDetailUseCase, checkIsPidUseCase, WalletMockData.card),
    act: (bloc) => bloc.add(CardDetailLoadTriggered(WalletMockData.card.attestationId!)),
    setUp: () {
      when(
        observeWalletCardDetailUseCase.invoke(WalletMockData.card.attestationId),
      ).thenAnswer((_) => Stream.error('Failed to load card details'));
    },
    expect: () => [CardDetailLoadFailure(WalletMockData.card.attestationId!)],
  );
}
