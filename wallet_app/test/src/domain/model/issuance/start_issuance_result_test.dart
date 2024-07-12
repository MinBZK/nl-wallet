import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/issuance/start_issuance_result.dart';

import '../../../mocks/wallet_mock_data.dart';

void main() {
  test('StartIssuanceReadyToDisclose', () {
    final StartIssuanceResult issuance = StartIssuanceReadyToDisclose(
      relyingParty: WalletMockData.organization,
      policy: WalletMockData.policy,
      requestedAttributes: {},
    );
    expect(issuance, isA<StartIssuanceReadyToDisclose>());
    expect(issuance.relyingParty, WalletMockData.organization);
    expect(issuance.policy, WalletMockData.policy);
  });

  test('StartIssuanceMissingAttributes', () {
    final StartIssuanceResult issuance = StartIssuanceMissingAttributes(
      relyingParty: WalletMockData.organization,
      policy: WalletMockData.policy,
      missingAttributes: [],
    );
    expect(issuance, isA<StartIssuanceMissingAttributes>());
    expect(issuance.relyingParty, WalletMockData.organization);
    expect(issuance.policy, WalletMockData.policy);
  });
}
