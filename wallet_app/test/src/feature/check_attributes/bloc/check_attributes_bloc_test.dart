import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/check_attributes/bloc/check_attributes_bloc.dart';

import '../../../mocks/wallet_mock_data.dart';

void main() {
  blocTest(
    'verify initial state',
    build: () => CheckAttributesBloc(cards: [WalletMockData.card, WalletMockData.altCard]),
    verify: (bloc) => expect(bloc.state, CheckAttributesInitial()),
  );

  blocTest(
    'verify initial state when providing a single card',
    build: () => CheckAttributesBloc.forCard(WalletMockData.card),
    verify: (bloc) => expect(
      bloc.state,
      CheckAttributesSuccess(card: WalletMockData.card, attributes: WalletMockData.card.attributes),
    ),
  );

  blocTest(
    'verify state when providing multiple cards and triggering load for the first card',
    build: () => CheckAttributesBloc(cards: [WalletMockData.card, WalletMockData.altCard]),
    act: (bloc) => bloc.add(CheckAttributesCardSelected(card: WalletMockData.altCard)),
    expect: () => [
      CheckAttributesSuccess(
        attributes: WalletMockData.altCard.attributes,
        card: WalletMockData.altCard,
        alternatives: [WalletMockData.card],
      ),
    ],
  );
}
