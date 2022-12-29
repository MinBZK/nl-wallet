part of 'mock_issuance_response_repository.dart';

const _kMockBirthDate = '10 maart 1997';
const _kMockBirthPlace = 'Delft';
const _kMockFirstNames = 'Willeke Liselotte';
const _kMockFullName = 'Willeke De Bruijn';
const _kMockLastName = 'De Bruijn';
const _kMockGender = 'Vrouw';

// region WalletCards
const _kPidId = 'PID_1';
const _kMockPidWalletCard = WalletCard(
  id: _kPidId,
  front: _kMockPidCardFront,
  attributes: _kMockPidDataAttributes,
);

const _kDiplomaId = 'DIPLOMA_1';
const _kMockDiplomaWalletCard = WalletCard(
  id: _kDiplomaId,
  front: _kMockDiplomaCardFront,
  attributes: _kMockDiplomaDataAttributes,
);

const _kMultiDiplomaId = 'MULTI_DIPLOMA';
const _kMasterDiplomaId = 'DIPLOMA_2';
const _kMockMasterDiplomaWalletCard = WalletCard(
  id: _kMasterDiplomaId,
  front: _kMockMasterDiplomaCardFront,
  attributes: _kMockMasterDiplomaDataAttributes,
);

const _kDrivingLicenseId = 'DRIVING_LICENSE';
final _kMockDrivingLicenseWalletCard = WalletCard(
  id: _kDrivingLicenseId,
  front: _kMockDrivingLicenseCardFront,
  attributes: _kMockDrivingLicenseDataAttributes,
);

const _kDrivingLicenseRenewedId = 'DRIVING_LICENSE_RENEWED'; // Used in issuance QR only!
final _kMockDrivingLicenseRenewedWalletCard = WalletCard(
  id: _kDrivingLicenseId, // Same id as initial license! Used to mock 'renewal' a.k.a. card update
  front: _kMockDrivingLicenseRenewedCardFront,
  attributes: _kMockDrivingLicenseRenewedDataAttributes,
);

const _kHealthInsuranceId = 'HEALTH_INSURANCE';
const _kMockHealthInsuranceWalletCard = WalletCard(
  id: _kHealthInsuranceId,
  front: _kMockHealthInsuranceCardFront,
  attributes: _kMockHealthInsuranceDataAttributes,
);

const _kVOGId = 'VOG';
const _kMockVOGWalletCard = WalletCard(
  id: _kVOGId,
  front: _kMockVOGCardFront,
  attributes: _kMockVOGDataAttributes,
);

// endregion

// region CardFronts

const _kMockPidCardFront = CardFront(
  title: 'Persoonsgegevens',
  subtitle: 'Willeke',
  logoImage: 'assets/non-free/images/logo_card_rijksoverheid.png',
  backgroundImage: 'assets/images/bg_pid.png',
  theme: CardFrontTheme.dark,
);

const _kMockDiplomaCardFront = CardFront(
  title: 'BSc. Diploma',
  info: 'Dienst Uitvoerend Onderwijs',
  logoImage: 'assets/non-free/images/logo_card_rijksoverheid.png',
  backgroundImage: 'assets/images/bg_diploma.png',
  theme: CardFrontTheme.dark,
);

const _kMockMasterDiplomaCardFront = CardFront(
  title: 'MSc. Diploma',
  info: 'Dienst Uitvoerend Onderwijs',
  logoImage: 'assets/non-free/images/logo_card_rijksoverheid.png',
  backgroundImage: 'assets/images/bg_diploma.png',
  theme: CardFrontTheme.dark,
);

const _kMockDrivingLicenseCardFront = CardFront(
  title: 'Rijbewijs',
  info: 'Categorie AM, B, BE',
  logoImage: 'assets/non-free/images/logo_nl_driving_license.png',
  backgroundImage: 'assets/images/bg_nl_driving_license.png',
  theme: CardFrontTheme.light,
);

const _kMockDrivingLicenseRenewedCardFront = CardFront(
  title: 'Rijbewijs',
  info: 'Categorie AM, B, C1, BE',
  logoImage: 'assets/non-free/images/logo_nl_driving_license.png',
  backgroundImage: 'assets/images/bg_nl_driving_license.png',
  theme: CardFrontTheme.light,
);

const _kMockHealthInsuranceCardFront = CardFront(
  title: 'European Health Insurance Card',
  subtitle: 'Zorgverzekeraar Z',
  logoImage: 'assets/non-free/images/logo_nl_health_insurance.png',
  backgroundImage: 'assets/images/bg_health_insurance.png',
  theme: CardFrontTheme.dark,
);

const _kMockVOGCardFront = CardFront(
  title: 'Verklaring Omtrent het Gedrag',
  info: 'Justis',
  logoImage: 'assets/non-free/images/logo_card_rijksoverheid.png',
  backgroundImage: 'assets/images/bg_diploma.png',
  theme: CardFrontTheme.dark,
);

// endregion

// region DataAttributes
const _kMockPidDataAttributes = [
  DataAttribute(
    valueType: AttributeValueType.image,
    label: 'Pasfoto',
    value: 'assets/non-free/images/person_x.png',
    type: AttributeType.profilePhoto,
    sourceCardId: _kPidId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Voornamen',
    value: _kMockFirstNames,
    type: AttributeType.firstNames,
    sourceCardId: _kPidId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Achternaam',
    value: _kMockLastName,
    type: AttributeType.lastName,
    sourceCardId: _kPidId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Geslachtsnaam',
    value: 'Molenaar',
    sourceCardId: _kPidId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Geslacht',
    value: _kMockGender,
    type: AttributeType.gender,
    sourceCardId: _kPidId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Geboortedatum',
    value: _kMockBirthDate,
    type: AttributeType.birthDate,
    sourceCardId: _kPidId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Ouder dan 18',
    value: 'Ja',
    type: AttributeType.olderThan18,
    sourceCardId: _kPidId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Geboorteplaats',
    value: _kMockBirthPlace,
    type: AttributeType.birthPlace,
    sourceCardId: _kPidId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Geboorteland',
    value: 'Nederland',
    type: AttributeType.birthCountry,
    sourceCardId: _kPidId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Burgerservicenummer (BSN)',
    value: '999999999',
    type: AttributeType.citizenshipNumber,
    sourceCardId: _kPidId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Woonplaats',
    value: 'Den Haag',
    type: AttributeType.city,
    sourceCardId: _kPidId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Postcode',
    value: '2511 DP',
    type: AttributeType.postalCode,
    sourceCardId: _kPidId,
  ),
];

const _kMockDiplomaDataAttributes = [
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Onderwijsinstelling',
    value: 'Universiteit X',
    type: AttributeType.university,
    sourceCardId: _kDiplomaId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Opleiding',
    value: 'WO Bachelor Bedrijfskunde',
    type: AttributeType.education,
    sourceCardId: _kDiplomaId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Niveau',
    value: 'WO',
    type: AttributeType.educationLevel,
    sourceCardId: _kDiplomaId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Type',
    value: 'Getuigschrift',
    sourceCardId: _kDiplomaId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Uitgifte datum',
    value: '1 januari 2013',
    type: AttributeType.issuanceDate,
    sourceCardId: _kDiplomaId,
  ),
];

const _kMockMasterDiplomaDataAttributes = [
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Onderwijsinstelling',
    value: 'Universiteit X',
    type: AttributeType.university,
    sourceCardId: _kMasterDiplomaId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Opleiding',
    value: 'WO Master Bedrijfskunde',
    type: AttributeType.education,
    sourceCardId: _kMasterDiplomaId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Niveau',
    value: 'WO',
    type: AttributeType.educationLevel,
    sourceCardId: _kMasterDiplomaId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Type',
    value: 'Getuigschrift',
    sourceCardId: _kMasterDiplomaId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Uitgifte datum',
    value: '1 januari 2015',
    type: AttributeType.issuanceDate,
    sourceCardId: _kMasterDiplomaId,
  ),
];

final _kMockDrivingLicenseDataAttributes = _buildDrivingLicenseDataAttributes(category: 'AM, B, BE');
final _kMockDrivingLicenseRenewedDataAttributes = _buildDrivingLicenseDataAttributes(category: 'AM, B, C1, BE');

List<DataAttribute> _buildDrivingLicenseDataAttributes({required String category}) {
  return [
    const DataAttribute(
      valueType: AttributeValueType.image,
      label: 'Pasfoto',
      value: 'assets/non-free/images/person_x.png',
      type: AttributeType.profilePhoto,
      sourceCardId: _kDrivingLicenseId,
    ),
    const DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Voornamen',
      value: _kMockFirstNames,
      type: AttributeType.firstNames,
      sourceCardId: _kDrivingLicenseId,
    ),
    const DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Naam',
      value: _kMockLastName,
      type: AttributeType.lastName,
      sourceCardId: _kDrivingLicenseId,
    ),
    const DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Geboortedatum',
      value: _kMockBirthDate,
      type: AttributeType.birthDate,
      sourceCardId: _kDrivingLicenseId,
    ),
    const DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Geboorteplaats',
      value: _kMockBirthPlace,
      type: AttributeType.birthPlace,
      sourceCardId: _kDrivingLicenseId,
    ),
    const DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Afgiftedatum',
      value: '23-04-2018',
      type: AttributeType.issuanceDate,
      sourceCardId: _kDrivingLicenseId,
    ),
    const DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Datum geldig tot',
      value: '23-04-2028',
      type: AttributeType.expiryDate,
      sourceCardId: _kDrivingLicenseId,
    ),
    const DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Rijbewijsnummer',
      value: '99999999999',
      sourceCardId: _kDrivingLicenseId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Rijbewijscategorieën',
      value: category,
      type: AttributeType.drivingLicenseCategories,
      sourceCardId: _kDrivingLicenseId,
    ),
  ];
}

const _kMockHealthInsuranceDataAttributes = [
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Naam',
    value: _kMockFullName,
    type: AttributeType.fullName,
    sourceCardId: _kHealthInsuranceId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Geslacht',
    value: _kMockGender,
    type: AttributeType.gender,
    sourceCardId: _kHealthInsuranceId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Geboortedatum',
    value: _kMockBirthDate,
    type: AttributeType.birthDate,
    sourceCardId: _kHealthInsuranceId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Klantnummer',
    value: '12345678',
    type: AttributeType.healthIssuerClientId,
    sourceCardId: _kHealthInsuranceId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Kaartnummer',
    value: '9999999999',
    type: AttributeType.documentNr,
    sourceCardId: _kHealthInsuranceId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'UZOVI',
    value: 'XXXX - 9999',
    type: AttributeType.healthIssuerId,
    sourceCardId: _kHealthInsuranceId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Verloopdatum',
    value: '1 januari 2024',
    type: AttributeType.expiryDate,
    sourceCardId: _kHealthInsuranceId,
  ),
];

const _kMockVOGDataAttributes = [
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Type',
    value: '1',
    type: AttributeType.certificateOfConduct,
    sourceCardId: _kVOGId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Datum geldig tot',
    value: '05-02-2023',
    type: AttributeType.expiryDate,
    sourceCardId: _kVOGId,
  ),
];

// endregion

// region RequestedAttributes
const _kMockGovernmentOrganizationRequestedAttributes = [
  RequestedAttribute(name: 'BSN', type: AttributeType.citizenshipNumber, valueType: AttributeValueType.text),
];

const _kMockHealthInsuranceRequestedAttributes = [
  RequestedAttribute(name: 'Voornamen', type: AttributeType.firstNames, valueType: AttributeValueType.text),
  RequestedAttribute(name: 'Achternaam', type: AttributeType.lastName, valueType: AttributeValueType.text),
  RequestedAttribute(name: 'Geboortedatum', type: AttributeType.birthDate, valueType: AttributeValueType.text),
];
// endregion

// region Policies

const _kMockIssuancePolicy = Policy(
  storageDuration: Duration(days: 90),
  dataPurpose: 'Kaart uitgifte',
  dataIsShared: false,
  dataIsSignature: false,
  deletionCanBeRequested: true,
  privacyPolicyUrl: 'https://www.example.org',
);

// endregion
