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
    requestPurpose: [LocalizedString(language: 'nl', value: '')],
    policy: _kMockIssuancePolicy,
    attestations: kPidAttestations,
  ),
  IssuanceResponse(
    id: 'DIPLOMA_1',
    relyingParty: kOrganizations[kDuoId]!,
    requestedAttributes: _kMockRequestBsnAttributes,
    requestPurpose: [LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    policy: _kMockIssuancePolicy,
    attestations: [kDiplomaAttestation],
  ),
  IssuanceResponse(
    id: 'MULTI_DIPLOMA',
    relyingParty: kOrganizations[kDuoId]!,
    requestedAttributes: _kMockRequestBsnAttributes,
    requestPurpose: [LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    policy: _kMockIssuancePolicy,
    attestations: [kDiplomaAttestation, kMockMasterDiplomaWalletCard],
  ),
  IssuanceResponse(
    id: 'DRIVING_LICENSE',
    relyingParty: kOrganizations[kRdwId]!,
    requestedAttributes: _kMockRequestBsnAttributes,
    requestPurpose: [LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    policy: _kMockIssuancePolicy,
    attestations: [kMockDrivingLicenseWalletCard],
  ),
  IssuanceResponse(
    id: 'DRIVING_LICENSE_RENEWED',
    relyingParty: kOrganizations[kRdwId]!,
    requestedAttributes: _kMockRequestBsnAttributes,
    requestPurpose: [LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    policy: _kMockIssuancePolicy,
    attestations: [kMockDrivingLicenseRenewedWalletCard],
  ),
  IssuanceResponse(
    id: 'HEALTH_INSURANCE',
    relyingParty: kOrganizations[kHealthInsuranceId]!,
    requestedAttributes: _kMockRequestNameDobAttributes,
    requestPurpose: [LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    policy: _kMockIssuancePolicy,
    attestations: [kMockHealthInsuranceWalletCard],
  ),
  IssuanceResponse(
    id: 'VOG',
    relyingParty: kOrganizations[kJusticeId]!,
    requestedAttributes: _kMockRequestBsnAttributes,
    requestPurpose: [LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    policy: _kMockIssuancePolicy,
    attestations: [kMockVOGWalletCard],
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

// region Policies

final _kMockIssuancePolicy = RequestPolicy(
  dataStorageDurationInMinutes: BigInt.from(60 * 24 * 90),
  dataSharedWithThirdParties: false,
  dataDeletionPossible: true,
  policyUrl: 'https://www.example.org',
);

// endregion
