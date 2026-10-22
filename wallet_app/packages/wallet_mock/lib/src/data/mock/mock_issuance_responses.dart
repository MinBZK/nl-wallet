import 'package:wallet_core/core.dart';

import '../model/issuance_response.dart';
import '../model/requested_attribute.dart';
import 'mock_attestations.dart';
import 'mock_attributes.dart';
import 'mock_organizations.dart';

const _kMockRequestPurpose = 'Kaart uitgifte';

final kIssuanceResponses = [
  IssuanceResponse(
    id: 'PID_1',
    relyingParty: kOrganizations[kRvigId]!,
    requestedAttributes: [],
    requestPurpose: [const LocalizedString(language: 'nl', value: '')],
    attestations: kPidAttestations,
  ),
  IssuanceResponse(
    id: 'DIPLOMA_1',
    relyingParty: kOrganizations[kDuoId]!,
    requestedAttributes: _kMockRequestBsnAttributes,
    requestPurpose: [const LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    attestations: [kDiplomaAttestation],
  ),
  IssuanceResponse(
    id: 'MULTI_DIPLOMA',
    relyingParty: kOrganizations[kDuoId]!,
    requestedAttributes: _kMockRequestBsnAttributes,
    requestPurpose: [const LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    attestations: [kDiplomaAttestation, kMockMasterDiplomaWalletCard],
  ),
  IssuanceResponse(
    id: 'DRIVING_LICENSE',
    relyingParty: kOrganizations[kRdwId]!,
    requestedAttributes: _kMockRequestBsnAttributes,
    requestPurpose: [const LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    attestations: [kMockDrivingLicenseWalletCard],
  ),
  IssuanceResponse(
    id: 'DRIVING_LICENSE_RENEWED',
    relyingParty: kOrganizations[kRdwId]!,
    requestedAttributes: _kMockRequestBsnAttributes,
    requestPurpose: [const LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    attestations: [kMockDrivingLicenseRenewedWalletCard],
  ),
  IssuanceResponse(
    id: 'HEALTH_INSURANCE',
    relyingParty: kOrganizations[kHealthInsuranceId]!,
    requestedAttributes: _kMockRequestNameDobAttributes,
    requestPurpose: [const LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    attestations: [kMockHealthInsuranceWalletCard],
  ),
  IssuanceResponse(
    id: 'VOG',
    relyingParty: kOrganizations[kJusticeId]!,
    requestedAttributes: _kMockRequestBsnAttributes,
    requestPurpose: [const LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    attestations: [kMockVOGWalletCard],
  ),
  IssuanceResponse(
    id: 'PRE-AUTH-FLOW',
    relyingParty: kOrganizations[kSupermarketId]!,
    requestedAttributes: [],
    requestPurpose: [const LocalizedString(language: 'nl', value: '')],
    attestations: [kMockLoyaltyCard],
  ),
  IssuanceResponse(
    id: 'AUTH-FLOW',
    relyingParty: kOrganizations[kSupermarketId]!,
    requestedAttributes: [],
    requestPurpose: [const LocalizedString(language: 'nl', value: 'Authenticatie')],
    attestations: [kMockLoyaltyCard],
  ),
];

// region RequestedAttributes
final _kMockRequestBsnAttributes = [RequestedAttribute(label: 'BSN', key: kMockCitizenShipNumberKey)];

final _kMockRequestNameDobAttributes = [
  RequestedAttribute(label: 'Voornamen', key: kMockFirstNamesKey),
  RequestedAttribute(label: 'Achternaam', key: kMockLastNameKey),
  RequestedAttribute(label: 'Geboortedatum', key: kMockBirthDateKey),
];
// endregion
