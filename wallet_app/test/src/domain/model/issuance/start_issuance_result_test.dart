import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_session_type.dart';
import 'package:wallet/src/domain/model/issuance/start_issuance_result.dart';
import 'package:wallet/src/util/extension/string_extension.dart';

import '../../../mocks/wallet_mock_data.dart';

void main() {
  test('Verify relyingParty getter works as expected for all variants', () {
    const authRequired = StartIssuanceResult.authorizationRequired('https://example.com');
    expect(authRequired.relyingParty, isNull);

    final preAuthOffer = StartIssuanceResult.preAuthorizedOffer([WalletMockData.card]);
    expect(preAuthOffer.relyingParty, WalletMockData.organization);

    final preAuthOfferEmpty = const StartIssuanceResult.preAuthorizedOffer([]);
    expect(preAuthOfferEmpty.relyingParty, isNull);

    final readyToDisclose = StartIssuanceResult.readyToDisclose(
      relyingParty: WalletMockData.organization,
      requestPurpose: 'purpose'.untranslated,
      sessionType: DisclosureSessionType.crossDevice,
      cardRequests: [],
      policy: WalletMockData.policy,
    );
    expect(readyToDisclose.relyingParty, WalletMockData.organization);

    final missingAttributes = StartIssuanceResult.missingAttributes(
      relyingParty: WalletMockData.organization,
      sessionType: DisclosureSessionType.crossDevice,
      missingAttributes: [],
    );
    expect(missingAttributes.relyingParty, WalletMockData.organization);
  });

  test('verify sessionType getter works as expected for all variants', () {
    const authRequired = StartIssuanceResult.authorizationRequired('https://example.com');
    expect(authRequired.sessionType, isNull);

    final preAuthOffer = StartIssuanceResult.preAuthorizedOffer([WalletMockData.card]);
    expect(preAuthOffer.sessionType, isNull);

    final readyToDisclose = StartIssuanceResult.readyToDisclose(
      relyingParty: WalletMockData.organization,
      requestPurpose: 'purpose'.untranslated,
      sessionType: DisclosureSessionType.crossDevice,
      cardRequests: [],
      policy: WalletMockData.policy,
    );
    expect(readyToDisclose.sessionType, DisclosureSessionType.crossDevice);

    final missingAttributes = StartIssuanceResult.missingAttributes(
      relyingParty: WalletMockData.organization,
      sessionType: DisclosureSessionType.sameDevice,
      missingAttributes: [],
    );
    expect(missingAttributes.sessionType, DisclosureSessionType.sameDevice);
  });
}
