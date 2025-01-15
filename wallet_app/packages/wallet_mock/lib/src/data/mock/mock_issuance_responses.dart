import 'package:wallet_core/core.dart';

import '../../../mock.dart';
import '../model/issuance_response.dart';
import '../model/requested_attribute.dart';
import 'mock_organizations.dart';

final _kMockBirthDate = CardValue.date(value: DateTime(1997, 3, 10).toIso8601String());
const _kMockBirthPlace = CardValue.string(value: 'Delft');
const _kMockFirstNames = CardValue.string(value: 'Willeke Liselotte');
const _kMockFullName = CardValue.string(value: 'Willeke De Bruijn');
const _kMockLastName = CardValue.string(value: 'De Bruijn');
const _kMockGender = CardValue.string(value: 'Vrouw');
const _kMockRequestPurpose = 'Kaart uitgifte';

const _kMockFirstNamesKey = 'mock.firstNames';
const _kMockLastNameKey = 'mock.lastName';
const _kMockBirthDateKey = 'mock.birthDate';
const _kMockOtherKey = 'mock.other';
const _kMockIssuanceDateKey = 'mock.issuanceDate';

final kIssuanceResponses = [
  IssuanceResponse(
    id: _kPidId,
    organization: kOrganizations[kRvigId]!,
    requestedAttributes: [],
    requestPurpose: [LocalizedString(language: 'nl', value: '')],
    policy: _kMockIssuancePolicy,
    cards: [_kMockPidWalletCard, _kMockAddressWalletCard],
  ),
  IssuanceResponse(
    id: _kDiplomaId,
    organization: kOrganizations[kDuoId]!,
    requestedAttributes: _kMockGovernmentOrganizationRequestedAttributes,
    requestPurpose: [LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    policy: _kMockIssuancePolicy,
    cards: [_kMockDiplomaWalletCard],
  ),
  IssuanceResponse(
    id: _kMultiDiplomaId,
    organization: kOrganizations[kDuoId]!,
    requestedAttributes: _kMockGovernmentOrganizationRequestedAttributes,
    requestPurpose: [LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    policy: _kMockIssuancePolicy,
    cards: [_kMockDiplomaWalletCard, _kMockMasterDiplomaWalletCard],
  ),
  IssuanceResponse(
    id: _kDrivingLicenseId,
    organization: kOrganizations[kRdwId]!,
    requestedAttributes: _kMockGovernmentOrganizationRequestedAttributes,
    requestPurpose: [LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    policy: _kMockIssuancePolicy,
    cards: [_kMockDrivingLicenseWalletCard],
  ),
  IssuanceResponse(
    id: _kDrivingLicenseRenewedId,
    organization: kOrganizations[kRdwId]!,
    requestedAttributes: _kMockGovernmentOrganizationRequestedAttributes,
    requestPurpose: [LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    policy: _kMockIssuancePolicy,
    cards: [_kMockDrivingLicenseRenewedWalletCard],
  ),
  IssuanceResponse(
    id: _kHealthInsuranceId,
    organization: kOrganizations[kHealthInsuranceId]!,
    requestedAttributes: _kMockHealthInsuranceRequestedAttributes,
    requestPurpose: [LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    policy: _kMockIssuancePolicy,
    cards: [_kMockHealthInsuranceWalletCard],
  ),
  IssuanceResponse(
    id: _kVOGId,
    organization: kOrganizations[kJusticeId]!,
    requestedAttributes: _kMockGovernmentOrganizationRequestedAttributes,
    requestPurpose: [LocalizedString(language: 'nl', value: _kMockRequestPurpose)],
    policy: _kMockIssuancePolicy,
    cards: [_kMockVOGWalletCard],
  ),
];

// region WalletCards
const _kPidId = 'PID_1';
final _kMockPidWalletCard = Card(
  docType: kPidDocType,
  persistence: CardPersistence.inMemory(),
  attributes: _kMockPidDataAttributes,
  issuer: kOrganizations[kRvigId]!,
);

final _kMockAddressWalletCard = Card(
  docType: kAddressDocType,
  persistence: CardPersistence.inMemory(),
  // front: _kMockAddressCardFront,
  attributes: _kMockAddressDataAttributes,
  issuer: kOrganizations[kRvigId]!,
);

const _kDiplomaId = 'DIPLOMA_1';
final _kMockDiplomaWalletCard = Card(
  docType: _kDiplomaId,
  persistence: CardPersistence.inMemory(),
  // front: _kMockDiplomaCardFront,
  attributes: _kMockDiplomaDataAttributes,
  issuer: kOrganizations[kDuoId]!,
);

const _kMultiDiplomaId = 'MULTI_DIPLOMA';
const _kMasterDiplomaId = 'DIPLOMA_2';
final _kMockMasterDiplomaWalletCard = Card(
  docType: _kMasterDiplomaId,
  persistence: CardPersistence.inMemory(),
  // front: _kMockMasterDiplomaCardFront,
  attributes: _kMockMasterDiplomaDataAttributes,
  issuer: kOrganizations[kDuoId]!,
);

const _kDrivingLicenseId = 'DRIVING_LICENSE';
final _kMockDrivingLicenseWalletCard = Card(
  docType: kDrivingLicenseDocType,
  persistence: CardPersistence.inMemory(),
  // front: _kMockDrivingLicenseCardFront,
  attributes: _kMockDrivingLicenseDataAttributes,
  issuer: kOrganizations[kRdwId]!,
);

const _kDrivingLicenseRenewedId = 'DRIVING_LICENSE_RENEWED'; // Used in issuance QR only!
final _kMockDrivingLicenseRenewedWalletCard = Card(
  docType: kDrivingLicenseDocType,
  persistence: CardPersistence.inMemory(),
  // front: _kMockDrivingLicenseRenewedCardFront,
  attributes: _kMockDrivingLicenseRenewedDataAttributes,
  issuer: kOrganizations[kRdwId]!,
);

const _kHealthInsuranceId = 'HEALTH_INSURANCE';
final _kMockHealthInsuranceWalletCard = Card(
  docType: _kHealthInsuranceId,
  persistence: CardPersistence.inMemory(),
  // front: _kMockHealthInsuranceCardFront,
  attributes: _kMockHealthInsuranceDataAttributes,
  issuer: kOrganizations[kHealthInsuranceId]!,
);

const _kVOGId = 'VOG';
final _kMockVOGWalletCard = Card(
  docType: _kVOGId,
  persistence: CardPersistence.inMemory(),
  // front: _kMockVOGCardFront,
  attributes: _kMockVOGDataAttributes,
  issuer: kOrganizations[kRvigId]!,
);

// endregion

// region DataAttributes
final _kMockPidDataAttributes = [
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Voornamen')],
    value: _kMockFirstNames,
    key: _kMockFirstNamesKey,
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Achternaam')],
    value: _kMockLastName,
    key: _kMockLastNameKey,
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Naam bij geboorte')],
    value: const CardValue.string(value: 'Molenaar'),
    key: 'mock.birthName',
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Geslacht')],
    value: _kMockGender,
    key: 'mock.gender',
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Geboortedatum')],
    value: _kMockBirthDate,
    key: _kMockBirthDateKey,
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Ouder dan 18')],
    value: CardValue.boolean(value: true),
    key: 'mock.olderThan18',
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Geboorteplaats')],
    value: _kMockBirthPlace,
    key: 'mock.birthPlace',
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Geboorteland')],
    value: const CardValue.string(value: 'Nederland'),
    key: 'mock.birthCountry',
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Burger­service­nummer (BSN)')],
    value: const CardValue.string(value: '111222333'),
    key: 'mock.citizenshipNumber',
  ),
];

final _kMockAddressDataAttributes = [
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Straatnaam')],
    value: const CardValue.string(value: 'Turfmarkt'),
    key: 'mock.streetName',
    // sourceCardId: _kAddressId,
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Huisnummer')],
    value: const CardValue.string(value: '147'),
    key: 'mock.houseNumber',
    // sourceCardId: _kAddressId,
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Postcode')],
    value: const CardValue.string(value: '2511 DP'),
    key: 'mock.postalCode',
    // sourceCardId: _kAddressId,
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Woonplaats')],
    value: const CardValue.string(value: 'Den Haag'),
    key: 'mock.city',
    // sourceCardId: _kAddressId,
  ),
];

final _kMockDiplomaDataAttributes = [
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Onderwijsinstelling')],
    value: const CardValue.string(value: 'Universiteit X'),
    key: 'mock.university',
    // sourceCardId: _kDiplomaId,
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Opleiding')],
    value: const CardValue.string(value: 'WO Bachelor Bedrijfskunde'),
    key: 'mock.education',
    // sourceCardId: _kDiplomaId,
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Niveau')],
    value: const CardValue.string(value: 'WO'),
    key: 'mock.educationLevel',
    // sourceCardId: _kDiplomaId,
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Type')],
    value: const CardValue.string(value: 'Getuigschrift'),
    // sourceCardId: _kDiplomaId,
    key: _kMockOtherKey,
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Uitgifte datum')],
    value: CardValue.date(value: DateTime(2013, 1, 1).toIso8601String()),
    key: _kMockIssuanceDateKey,
    // sourceCardId: _kDiplomaId,
  ),
];

final _kMockMasterDiplomaDataAttributes = [
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Onderwijsinstelling')],
    value: const CardValue.string(value: 'Universiteit X'),
    key: 'mock.university',
    // sourceCardId: _kMasterDiplomaId,
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Opleiding')],
    value: const CardValue.string(value: 'WO Master Bedrijfskunde'),
    key: 'mock.education',
    // sourceCardId: _kMasterDiplomaId,
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Niveau')],
    value: const CardValue.string(value: 'WO'),
    key: 'mock.educationLevel',
    // sourceCardId: _kMasterDiplomaId,
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Type')],
    value: const CardValue.string(value: 'Getuigschrift'),
    // sourceCardId: _kMasterDiplomaId,
    key: _kMockOtherKey,
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Uitgifte datum')],
    value: CardValue.date(value: DateTime(2015, 1, 1).toIso8601String()),
    key: _kMockIssuanceDateKey,
    // sourceCardId: _kMasterDiplomaId,
  ),
];

final _kMockDrivingLicenseDataAttributes = _buildDrivingLicenseDataAttributes(category: 'AM, B, BE');
final _kMockDrivingLicenseRenewedDataAttributes = _buildDrivingLicenseDataAttributes(category: 'AM, B, C1, BE');

List<CardAttribute> _buildDrivingLicenseDataAttributes({required String category}) {
  return [
    CardAttribute(
      labels: [LocalizedString(language: 'nl', value: 'Voornamen')],
      value: _kMockFirstNames,
      key: _kMockFirstNamesKey,
      // sourceCardId: _kDrivingLicenseId,
    ),
    CardAttribute(
      labels: [LocalizedString(language: 'nl', value: 'Naam')],
      value: _kMockLastName,
      key: _kMockLastNameKey,
      // sourceCardId: _kDrivingLicenseId,
    ),
    CardAttribute(
      labels: [LocalizedString(language: 'nl', value: 'Geboortedatum')],
      value: _kMockBirthDate,
      key: _kMockBirthDateKey,
      // sourceCardId: _kDrivingLicenseId,
    ),
    CardAttribute(
      labels: [LocalizedString(language: 'nl', value: 'Geboorteplaats')],
      value: _kMockBirthPlace,
      key: 'mock.birthPlace',
      // sourceCardId: _kDrivingLicenseId,
    ),
    CardAttribute(
      labels: [LocalizedString(language: 'nl', value: 'Afgiftedatum')],
      value: CardValue.date(value: DateTime(2018, 4, 23).toIso8601String()),
      key: _kMockIssuanceDateKey,
      // sourceCardId: _kDrivingLicenseId,
    ),
    CardAttribute(
      labels: [LocalizedString(language: 'nl', value: 'Datum geldig tot')],
      value: CardValue.date(value: DateTime(2028, 4, 23).toIso8601String()),
      key: 'mock.expiryDate',
      // sourceCardId: _kDrivingLicenseId,
    ),
    CardAttribute(
      labels: [LocalizedString(language: 'nl', value: 'Rijbewijsnummer')],
      value: const CardValue.string(value: '99999999999'),
      // sourceCardId: _kDrivingLicenseId,
      key: _kMockOtherKey,
    ),
    CardAttribute(
      labels: [LocalizedString(language: 'nl', value: 'Rijbewijscategorieën')],
      value: CardValue.string(value: category),
      key: 'mock.drivingLicenseCategories',
      // sourceCardId: _kDrivingLicenseId,
    ),
  ];
}

final _kMockHealthInsuranceDataAttributes = [
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Naam')],
    value: _kMockFullName,
    key: 'mock.fullName',
    // sourceCardId: _kHealthInsuranceId,
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Geslacht')],
    value: _kMockGender,
    key: 'mock.gender',
    // sourceCardId: _kHealthInsuranceId,
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Geboortedatum')],
    value: _kMockBirthDate,
    key: _kMockBirthDateKey,
    // sourceCardId: _kHealthInsuranceId,
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Klantnummer')],
    value: const CardValue.string(value: '12345678'),
    key: 'mock.healthIssuerClientId',
    // sourceCardId: _kHealthInsuranceId,
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Kaartnummer')],
    value: const CardValue.string(value: '9999999999'),
    key: 'mock.documentNr',
    // sourceCardId: _kHealthInsuranceId,
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'UZOVI')],
    value: const CardValue.string(value: 'XXXX - 9999'),
    key: 'mock.healthIssuerId',
    // sourceCardId: _kHealthInsuranceId,
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Verloopdatum')],
    value: CardValue.date(value: DateTime(2024, 1, 1).toIso8601String()),
    key: 'mock.healthInsuranceExpiryDate',
    // sourceCardId: _kHealthInsuranceId,
  ),
];

final _kMockVOGDataAttributes = [
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Type')],
    value: const CardValue.string(value: '1'),
    key: 'mock.certificateOfConduct',
    // sourceCardId: _kVOGId,
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Datum geldig tot')],
    value: CardValue.date(value: DateTime(2023, 2, 5).toIso8601String()),
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

const _kMockIssuancePolicy = RequestPolicy(
  dataStorageDurationInMinutes: 60 * 24 * 90,
  dataSharedWithThirdParties: false,
  dataDeletionPossible: true,
  policyUrl: 'https://www.example.org',
);

// endregion
