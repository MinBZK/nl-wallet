import 'package:wallet_core/core.dart';

import '../../../mock.dart';
import '../model/issuance_response.dart';
import '../model/requested_attribute.dart';
import 'mock_organizations.dart';

final _kMockBirthDate = AttributeValue.string(value: '10-03-1997');
const _kMockBirthPlace = AttributeValue.string(value: 'Delft');
const _kMockFirstNames = AttributeValue.string(value: 'Willeke Liselotte');
const _kMockFullName = AttributeValue.string(value: 'Willeke De Bruijn');
const _kMockLastName = AttributeValue.string(value: 'De Bruijn');
const _kMockGender = AttributeValue.string(value: 'Vrouw');
const _kMockRequestPurpose = 'Kaart uitgifte';

const _kMockFirstNamesKey = 'mock.firstNames';
const _kMockLastNameKey = 'mock.lastName';
const _kMockBirthDateKey = 'mock.birthDate';
const _kMockOtherKey = 'mock.other';
const _kMockIssuanceDateKey = 'mock.issuanceDate';

const _kMockDisplayMetadata = DisplayMetadata(lang: 'nl', name: 'card name');

final kIssuanceResponses = [
  IssuanceResponse(
    id: _kPidId,
    organization: kOrganizations[kRvigId]!,
    requestedAttributes: [],
    requestPurpose: [LocalizedString(language: 'nl', value: '')],
    policy: _kMockIssuancePolicy,
    attestations: [_kMockPidWalletCard, _kMockAddressWalletCard],
  ),
  IssuanceResponse(
    id: _kDiplomaId,
    organization: kOrganizations[kDuoId]!,
    requestedAttributes: _kMockGovernmentOrganizationRequestedAttributes,
    requestPurpose: [LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    policy: _kMockIssuancePolicy,
    attestations: [_kMockDiplomaWalletCard],
  ),
  IssuanceResponse(
    id: _kMultiDiplomaId,
    organization: kOrganizations[kDuoId]!,
    requestedAttributes: _kMockGovernmentOrganizationRequestedAttributes,
    requestPurpose: [LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    policy: _kMockIssuancePolicy,
    attestations: [_kMockDiplomaWalletCard, _kMockMasterDiplomaWalletCard],
  ),
  IssuanceResponse(
    id: _kDrivingLicenseId,
    organization: kOrganizations[kRdwId]!,
    requestedAttributes: _kMockGovernmentOrganizationRequestedAttributes,
    requestPurpose: [LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    policy: _kMockIssuancePolicy,
    attestations: [_kMockDrivingLicenseWalletCard],
  ),
  IssuanceResponse(
    id: _kDrivingLicenseRenewedId,
    organization: kOrganizations[kRdwId]!,
    requestedAttributes: _kMockGovernmentOrganizationRequestedAttributes,
    requestPurpose: [LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    policy: _kMockIssuancePolicy,
    attestations: [_kMockDrivingLicenseRenewedWalletCard],
  ),
  IssuanceResponse(
    id: _kHealthInsuranceId,
    organization: kOrganizations[kHealthInsuranceId]!,
    requestedAttributes: _kMockHealthInsuranceRequestedAttributes,
    requestPurpose: [LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    policy: _kMockIssuancePolicy,
    attestations: [_kMockHealthInsuranceWalletCard],
  ),
  IssuanceResponse(
    id: _kVOGId,
    organization: kOrganizations[kJusticeId]!,
    requestedAttributes: _kMockGovernmentOrganizationRequestedAttributes,
    requestPurpose: [LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    policy: _kMockIssuancePolicy,
    attestations: [_kMockVOGWalletCard],
  ),
];

// region WalletCards
const _kPidId = 'PID_1';
final _kMockPidWalletCard = Attestation(
  attestationType: kPidDocType,
  identity: AttestationIdentity.ephemeral(),
  displayMetadata: [_kMockDisplayMetadata],
  attributes: _kMockPidDataAttributes,
  issuer: kOrganizations[kRvigId]!,
);

final _kMockAddressWalletCard = Attestation(
  attestationType: kAddressDocType,
  identity: AttestationIdentity.ephemeral(),
  displayMetadata: [_kMockDisplayMetadata],
  // front: _kMockAddressCardFront,
  attributes: _kMockAddressDataAttributes,
  issuer: kOrganizations[kRvigId]!,
);

const _kDiplomaId = 'DIPLOMA_1';
final _kMockDiplomaWalletCard = Attestation(
  attestationType: _kDiplomaId,
  identity: AttestationIdentity.ephemeral(),
  displayMetadata: [_kMockDisplayMetadata],
  // front: _kMockDiplomaCardFront,
  attributes: _kMockDiplomaDataAttributes,
  issuer: kOrganizations[kDuoId]!,
);

const _kMultiDiplomaId = 'MULTI_DIPLOMA';
const _kMasterDiplomaId = 'DIPLOMA_2';
final _kMockMasterDiplomaWalletCard = Attestation(
  attestationType: _kMasterDiplomaId,
  identity: AttestationIdentity.ephemeral(),
  displayMetadata: [_kMockDisplayMetadata],
  // front: _kMockMasterDiplomaCardFront,
  attributes: _kMockMasterDiplomaDataAttributes,
  issuer: kOrganizations[kDuoId]!,
);

const _kDrivingLicenseId = 'DRIVING_LICENSE';
final _kMockDrivingLicenseWalletCard = Attestation(
  attestationType: kDrivingLicenseDocType,
  identity: AttestationIdentity.ephemeral(),
  displayMetadata: [_kMockDisplayMetadata],
  // front: _kMockDrivingLicenseCardFront,
  attributes: _kMockDrivingLicenseDataAttributes,
  issuer: kOrganizations[kRdwId]!,
);

const _kDrivingLicenseRenewedId = 'DRIVING_LICENSE_RENEWED'; // Used in issuance QR only!
final _kMockDrivingLicenseRenewedWalletCard = Attestation(
  attestationType: kDrivingLicenseDocType,
  identity: AttestationIdentity.ephemeral(),
  displayMetadata: [_kMockDisplayMetadata],
  // front: _kMockDrivingLicenseRenewedCardFront,
  attributes: _kMockDrivingLicenseRenewedDataAttributes,
  issuer: kOrganizations[kRdwId]!,
);

const _kHealthInsuranceId = 'HEALTH_INSURANCE';
final _kMockHealthInsuranceWalletCard = Attestation(
  attestationType: _kHealthInsuranceId,
  identity: AttestationIdentity.ephemeral(),
  displayMetadata: [_kMockDisplayMetadata],
  // front: _kMockHealthInsuranceCardFront,
  attributes: _kMockHealthInsuranceDataAttributes,
  issuer: kOrganizations[kHealthInsuranceId]!,
);

const _kVOGId = 'VOG';
final _kMockVOGWalletCard = Attestation(
  attestationType: _kVOGId,
  identity: AttestationIdentity.ephemeral(),
  displayMetadata: [_kMockDisplayMetadata],
  // front: _kMockVOGCardFront,
  attributes: _kMockVOGDataAttributes,
  issuer: kOrganizations[kRvigId]!,
);

// endregion

// region DataAttributes
final _kMockPidDataAttributes = [
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Voornamen')],
    value: _kMockFirstNames,
    key: _kMockFirstNamesKey,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Achternaam')],
    value: _kMockLastName,
    key: _kMockLastNameKey,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Naam bij geboorte')],
    value: const AttributeValue.string(value: 'Molenaar'),
    key: 'mock.birthName',
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Geslacht')],
    value: _kMockGender,
    key: 'mock.gender',
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Geboortedatum')],
    value: _kMockBirthDate,
    key: _kMockBirthDateKey,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Ouder dan 18')],
    value: AttributeValue.boolean(value: true),
    key: 'mock.olderThan18',
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Geboorteplaats')],
    value: _kMockBirthPlace,
    key: 'mock.birthPlace',
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Geboorteland')],
    value: const AttributeValue.string(value: 'Nederland'),
    key: 'mock.birthCountry',
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Burger­service­nummer (BSN)')],
    value: const AttributeValue.string(value: '111222333'),
    key: 'mock.citizenshipNumber',
  ),
];

final _kMockAddressDataAttributes = [
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Straatnaam')],
    value: const AttributeValue.string(value: 'Turfmarkt'),
    key: 'mock.streetName',
    // sourceCardId: _kAddressId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Huisnummer')],
    value: const AttributeValue.string(value: '147'),
    key: 'mock.houseNumber',
    // sourceCardId: _kAddressId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Postcode')],
    value: const AttributeValue.string(value: '2511 DP'),
    key: 'mock.postalCode',
    // sourceCardId: _kAddressId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Woonplaats')],
    value: const AttributeValue.string(value: 'Den Haag'),
    key: 'mock.city',
    // sourceCardId: _kAddressId,
  ),
];

final _kMockDiplomaDataAttributes = [
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Onderwijsinstelling')],
    value: const AttributeValue.string(value: 'Universiteit X'),
    key: 'mock.university',
    // sourceCardId: _kDiplomaId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Opleiding')],
    value: const AttributeValue.string(value: 'WO Bachelor Bedrijfskunde'),
    key: 'mock.education',
    // sourceCardId: _kDiplomaId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Niveau')],
    value: const AttributeValue.string(value: 'WO'),
    key: 'mock.educationLevel',
    // sourceCardId: _kDiplomaId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Type')],
    value: const AttributeValue.string(value: 'Getuigschrift'),
    // sourceCardId: _kDiplomaId,
    key: _kMockOtherKey,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Uitgifte datum')],
    value: AttributeValue.string(value: '01-01-2013'),
    key: _kMockIssuanceDateKey,
    // sourceCardId: _kDiplomaId,
  ),
];

final _kMockMasterDiplomaDataAttributes = [
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Onderwijsinstelling')],
    value: const AttributeValue.string(value: 'Universiteit X'),
    key: 'mock.university',
    // sourceCardId: _kMasterDiplomaId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Opleiding')],
    value: const AttributeValue.string(value: 'WO Master Bedrijfskunde'),
    key: 'mock.education',
    // sourceCardId: _kMasterDiplomaId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Niveau')],
    value: const AttributeValue.string(value: 'WO'),
    key: 'mock.educationLevel',
    // sourceCardId: _kMasterDiplomaId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Type')],
    value: const AttributeValue.string(value: 'Getuigschrift'),
    // sourceCardId: _kMasterDiplomaId,
    key: _kMockOtherKey,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Uitgifte datum')],
    value: AttributeValue.string(value: '01-01-2015'),
    key: _kMockIssuanceDateKey,
    // sourceCardId: _kMasterDiplomaId,
  ),
];

final _kMockDrivingLicenseDataAttributes = _buildDrivingLicenseDataAttributes(category: 'AM, B, BE');
final _kMockDrivingLicenseRenewedDataAttributes = _buildDrivingLicenseDataAttributes(category: 'AM, B, C1, BE');

List<AttestationAttribute> _buildDrivingLicenseDataAttributes({required String category}) {
  return [
    AttestationAttribute(
      labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Voornamen')],
      value: _kMockFirstNames,
      key: _kMockFirstNamesKey,
      // sourceCardId: _kDrivingLicenseId,
    ),
    AttestationAttribute(
      labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Naam')],
      value: _kMockLastName,
      key: _kMockLastNameKey,
      // sourceCardId: _kDrivingLicenseId,
    ),
    AttestationAttribute(
      labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Geboortedatum')],
      value: _kMockBirthDate,
      key: _kMockBirthDateKey,
      // sourceCardId: _kDrivingLicenseId,
    ),
    AttestationAttribute(
      labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Geboorteplaats')],
      value: _kMockBirthPlace,
      key: 'mock.birthPlace',
      // sourceCardId: _kDrivingLicenseId,
    ),
    AttestationAttribute(
      labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Afgiftedatum')],
      value: AttributeValue.string(value: '04-23-2018'),
      key: _kMockIssuanceDateKey,
      // sourceCardId: _kDrivingLicenseId,
    ),
    AttestationAttribute(
      labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Datum geldig tot')],
      value: AttributeValue.string(value: '23-04-2028'),
      key: 'mock.expiryDate',
      // sourceCardId: _kDrivingLicenseId,
    ),
    AttestationAttribute(
      labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Rijbewijsnummer')],
      value: const AttributeValue.string(value: '99999999999'),
      // sourceCardId: _kDrivingLicenseId,
      key: _kMockOtherKey,
    ),
    AttestationAttribute(
      labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Rijbewijscategorieën')],
      value: AttributeValue.string(value: category),
      key: 'mock.drivingLicenseCategories',
      // sourceCardId: _kDrivingLicenseId,
    ),
  ];
}

final _kMockHealthInsuranceDataAttributes = [
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Naam')],
    value: _kMockFullName,
    key: 'mock.fullName',
    // sourceCardId: _kHealthInsuranceId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Geslacht')],
    value: _kMockGender,
    key: 'mock.gender',
    // sourceCardId: _kHealthInsuranceId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Geboortedatum')],
    value: _kMockBirthDate,
    key: _kMockBirthDateKey,
    // sourceCardId: _kHealthInsuranceId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Klantnummer')],
    value: const AttributeValue.string(value: '12345678'),
    key: 'mock.healthIssuerClientId',
    // sourceCardId: _kHealthInsuranceId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Kaartnummer')],
    value: const AttributeValue.string(value: '9999999999'),
    key: 'mock.documentNr',
    // sourceCardId: _kHealthInsuranceId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'UZOVI')],
    value: const AttributeValue.string(value: 'XXXX - 9999'),
    key: 'mock.healthIssuerId',
    // sourceCardId: _kHealthInsuranceId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Verloopdatum')],
    value: AttributeValue.string(value: '0-01-2024'),
    key: 'mock.healthInsuranceExpiryDate',
    // sourceCardId: _kHealthInsuranceId,
  ),
];

final _kMockVOGDataAttributes = [
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Type')],
    value: const AttributeValue.string(value: '1'),
    key: 'mock.certificateOfConduct',
    // sourceCardId: _kVOGId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Datum geldig tot')],
    value: AttributeValue.string(value: '05-02-2023'),
    key: 'mock.expiryDate',
    // sourceCardId: _kVOGId,
  ),
];

// endregion

// region RequestedAttributes
final _kMockGovernmentOrganizationRequestedAttributes = [
  RequestedAttribute(
    label: 'BSN',
    key: 'mock.citizenshipNumber',
  ),
];

final _kMockHealthInsuranceRequestedAttributes = [
  RequestedAttribute(
    label: 'Voornamen',
    key: _kMockFirstNamesKey,
  ),
  RequestedAttribute(
    label: 'Achternaam',
    key: _kMockLastNameKey,
  ),
  RequestedAttribute(
    label: 'Geboortedatum',
    key: _kMockBirthDateKey,
  ),
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
