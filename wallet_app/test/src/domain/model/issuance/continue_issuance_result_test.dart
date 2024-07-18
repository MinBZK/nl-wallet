import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/issuance/continue_issuance_result.dart';

import '../../../mocks/wallet_mock_data.dart';

void main() {
  test('ContinueIssuanceResult', () {
    final result = ContinueIssuanceResult([WalletMockData.card]);
    expect(result.cards, [WalletMockData.card]);
  });
}
