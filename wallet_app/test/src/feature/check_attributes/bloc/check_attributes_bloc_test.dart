import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/check_attributes/bloc/check_attributes_bloc.dart';

import '../../../mocks/wallet_mock_data.dart';

void main() {
  blocTest(
    'verify initial state',
    build: () => CheckAttributesBloc(
      attributes: WalletMockData.card.attributes,
      card: WalletMockData.card,
    ),
    verify: (bloc) {
      expect(
        bloc.state,
        CheckAttributesInitial(
          attributes: WalletMockData.card.attributes,
          card: WalletMockData.card,
        ),
      );
    },
  );

  blocTest(
    'verify success state',
    build: () => CheckAttributesBloc(
      attributes: WalletMockData.card.attributes,
      card: WalletMockData.card,
    ),
    act: (bloc) => bloc.add(CheckAttributesLoadTriggered()),
    expect: () => [
      CheckAttributesSuccess(
        attributes: WalletMockData.card.attributes,
        card: WalletMockData.card,
      ),
    ],
  );
}
