import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_session_type.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_type.dart';
import 'package:wallet/src/domain/model/issuance/start_issuance_result.dart';
import 'package:wallet/src/util/extension/string_extension.dart';

import '../../../mocks/wallet_mock_data.dart';

void main() {
  test('StartIssuanceReadyToDisclose', () {
    final StartIssuanceResult issuance = StartIssuanceReadyToDisclose(
      relyingParty: WalletMockData.organization,
      policy: WalletMockData.policy,
      sessionType: DisclosureSessionType.crossDevice,
      cardRequests: [],
      originUrl: 'url',
      requestPurpose: 'test'.untranslated,
      type: DisclosureType.regular,
      sharedDataWithOrganizationBefore: false,
    );
    expect(issuance, isA<StartIssuanceReadyToDisclose>());
    expect(issuance.relyingParty, WalletMockData.organization);
    expect(issuance.sessionType, DisclosureSessionType.crossDevice);
    expect(issuance.originUrl, 'url');
    expect(issuance.requestPurpose, 'test'.untranslated);
    expect(issuance.sharedDataWithOrganizationBefore, isFalse);
  });

  test('StartIssuanceMissingAttributes', () {
    final StartIssuanceResult issuance = StartIssuanceMissingAttributes(
      relyingParty: WalletMockData.organization,
      sessionType: DisclosureSessionType.sameDevice,
      missingAttributes: [],
      originUrl: 'originUrl',
      requestPurpose: 'test'.untranslated,
      sharedDataWithOrganizationBefore: true,
    );
    expect(issuance, isA<StartIssuanceMissingAttributes>());
    expect(issuance.relyingParty, WalletMockData.organization);
    expect(issuance.originUrl, 'originUrl');
    expect(issuance.sessionType, DisclosureSessionType.sameDevice);
    expect(issuance.requestPurpose, 'test'.untranslated);
    expect(issuance.sharedDataWithOrganizationBefore, isTrue);
  });
}
