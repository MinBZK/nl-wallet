import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_session_type.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_type.dart';
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
      originUrl: 'https://example.com',
      requestPurpose: 'purpose'.untranslated,
      sessionType: DisclosureSessionType.crossDevice,
      type: DisclosureType.regular,
      cardRequests: [],
      policy: WalletMockData.policy,
      sharedDataWithOrganizationBefore: false,
    );
    expect(readyToDisclose.relyingParty, WalletMockData.organization);

    final missingAttributes = StartIssuanceResult.missingAttributes(
      relyingParty: WalletMockData.organization,
      originUrl: 'https://example.com',
      requestPurpose: 'purpose'.untranslated,
      sessionType: DisclosureSessionType.crossDevice,
      missingAttributes: [],
      sharedDataWithOrganizationBefore: false,
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
      originUrl: 'https://example.com',
      requestPurpose: 'purpose'.untranslated,
      sessionType: DisclosureSessionType.crossDevice,
      type: DisclosureType.regular,
      cardRequests: [],
      policy: WalletMockData.policy,
      sharedDataWithOrganizationBefore: false,
    );
    expect(readyToDisclose.sessionType, DisclosureSessionType.crossDevice);

    final missingAttributes = StartIssuanceResult.missingAttributes(
      relyingParty: WalletMockData.organization,
      originUrl: 'https://example.com',
      requestPurpose: 'purpose'.untranslated,
      sessionType: DisclosureSessionType.sameDevice,
      missingAttributes: [],
      sharedDataWithOrganizationBefore: false,
    );
    expect(missingAttributes.sessionType, DisclosureSessionType.sameDevice);
  });
}
