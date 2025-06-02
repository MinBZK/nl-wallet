import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/feature/issuance/bloc/issuance_bloc.dart';

import '../../../mocks/wallet_mock_data.dart';

void main() {
  test('IssuanceInitiated equals works', () {
    final actual = const IssuanceSessionStarted('test');
    final equal = const IssuanceSessionStarted('test');
    final diff = const IssuanceSessionStarted('other');
    expect(actual, equals(equal));
    expect(actual, isNot(diff));
  });

  test('IssuanceApproveCards equals works', () {
    final actual = IssuanceApproveCards(cards: [WalletMockData.card, WalletMockData.altCard]);
    final equal = IssuanceApproveCards(cards: [WalletMockData.card, WalletMockData.altCard]);
    final diff = IssuanceApproveCards(cards: [WalletMockData.card]);
    expect(actual, equals(equal));
    expect(actual, isNot(diff));
  });

  test('IssuanceConfirmPinFailed equals works', () {
    final actual = const IssuanceConfirmPinFailed(error: GenericError('test', sourceError: 'test'));
    final equal = const IssuanceConfirmPinFailed(error: GenericError('test', sourceError: 'test'));
    final diff = const IssuanceConfirmPinFailed(error: GenericError('alternative', sourceError: 'alternative'));
    expect(actual, equals(equal));
    expect(actual, isNot(diff));
  });

  test('IssuanceCardToggled equals works', () {
    final actual = IssuanceCardToggled(WalletMockData.card);
    final equal = IssuanceCardToggled(WalletMockData.card);
    final diff = IssuanceCardToggled(WalletMockData.altCard);
    expect(actual, equals(equal));
    expect(actual, isNot(diff));
  });
}
