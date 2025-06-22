import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/usecase/card/observe_wallet_card_usecase.dart';
import 'package:wallet/src/feature/card/data/bloc/card_data_bloc.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late ObserveWalletCardUseCase usecase;
  late CardDataBloc bloc;

  setUp(() {
    usecase = MockObserveWalletCardUseCase();
    bloc = CardDataBloc(usecase);
  });

  blocTest<CardDataBloc, CardDataState>(
    'bloc emits loading followed by the requested card',
    build: () => bloc,
    setUp: () {
      when(usecase.invoke(WalletMockData.card.attestationId!)).thenAnswer(
        (_) => Stream.value(WalletMockData.card),
      );
    },
    act: (bloc) async => bloc.add(CardDataLoadTriggered(WalletMockData.card.attestationId!)),
    expect: () async => [
      const CardDataLoadInProgress(),
      CardDataLoadSuccess(WalletMockData.card),
    ],
  );

  blocTest<CardDataBloc, CardDataState>(
    "bloc emits failure when card can't be loaded",
    build: () => bloc,
    setUp: () {
      when(usecase.invoke(WalletMockData.card.attestationId!)).thenAnswer(
        (_) => Stream.error('no card'),
      );
    },
    act: (bloc) async => bloc.add(CardDataLoadTriggered(WalletMockData.card.attestationId!)),
    expect: () async => [const CardDataLoadInProgress(), const CardDataLoadFailure()],
  );
}
