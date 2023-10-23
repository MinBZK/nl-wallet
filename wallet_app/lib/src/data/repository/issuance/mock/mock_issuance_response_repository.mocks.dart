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
  issuerId: kRvigId,
  front: _kMockPidCardFront,
  attributes: _kMockPidDataAttributes,
  config: CardConfig(),
);

const _kAddressId = 'ADDRESS_1';
const _kMockAddressWalletCard = WalletCard(
  id: _kAddressId,
  issuerId: kRvigId,
  front: _kMockAddressCardFront,
  attributes: _kMockAddressDataAttributes,
);

const _kDiplomaId = 'DIPLOMA_1';
const _kMockDiplomaWalletCard = WalletCard(
  id: _kDiplomaId,
  issuerId: kDuoId,
  front: _kMockDiplomaCardFront,
  attributes: _kMockDiplomaDataAttributes,
);

const _kMultiDiplomaId = 'MULTI_DIPLOMA';
const _kMasterDiplomaId = 'DIPLOMA_2';
const _kMockMasterDiplomaWalletCard = WalletCard(
  id: _kMasterDiplomaId,
  issuerId: kDuoId,
  front: _kMockMasterDiplomaCardFront,
  attributes: _kMockMasterDiplomaDataAttributes,
);

const _kDrivingLicenseId = 'DRIVING_LICENSE';
final _kMockDrivingLicenseWalletCard = WalletCard(
  id: _kDrivingLicenseId,
  issuerId: kRdwId,
  front: _kMockDrivingLicenseCardFront,
  attributes: _kMockDrivingLicenseDataAttributes,
);

const _kDrivingLicenseRenewedId = 'DRIVING_LICENSE_RENEWED'; // Used in issuance QR only!
final _kMockDrivingLicenseRenewedWalletCard = WalletCard(
  id: _kDrivingLicenseId, // Same id as initial license! Used to mock 'renewal' a.k.a. card update
  issuerId: kRdwId,
  front: _kMockDrivingLicenseRenewedCardFront,
  attributes: _kMockDrivingLicenseRenewedDataAttributes,
);

const _kHealthInsuranceId = 'HEALTH_INSURANCE';
const _kMockHealthInsuranceWalletCard = WalletCard(
  id: _kHealthInsuranceId,
  issuerId: kHealthInsuranceId,
  front: _kMockHealthInsuranceCardFront,
  attributes: _kMockHealthInsuranceDataAttributes,
);

const _kVOGId = 'VOG';
const _kMockVOGWalletCard = WalletCard(
  id: _kVOGId,
  issuerId: kJusticeId,
  front: _kMockVOGCardFront,
  attributes: _kMockVOGDataAttributes,
);

// endregion

// region CardFronts

const _kMockPidCardFront = CardFront(
  title: 'Persoonsgegevens',
  subtitle: 'Willeke',
  logoImage: WalletAssets.logo_card_rijksoverheid,
  holoImage: WalletAssets.svg_rijks_card_holo,
  backgroundImage: WalletAssets.svg_rijks_card_bg_light,
  theme: CardFrontTheme.light,
);

const _kMockAddressCardFront = CardFront(
  title: 'Woonadres',
  subtitle: "'s-Gravenhage",
  logoImage: WalletAssets.logo_card_rijksoverheid,
  holoImage: WalletAssets.svg_rijks_card_holo,
  backgroundImage: WalletAssets.svg_rijks_card_bg_dark,
  theme: CardFrontTheme.dark,
);

const _kMockDiplomaCardFront = CardFront(
  title: 'BSc. Diploma',
  info: 'Dienst Uitvoerend Onderwijs',
  logoImage: WalletAssets.logo_card_rijksoverheid,
  backgroundImage: WalletAssets.image_bg_diploma,
  theme: CardFrontTheme.dark,
);

const _kMockMasterDiplomaCardFront = CardFront(
  title: 'MSc. Diploma',
  info: 'Dienst Uitvoerend Onderwijs',
  logoImage: WalletAssets.logo_card_rijksoverheid,
  backgroundImage: WalletAssets.image_bg_diploma,
  theme: CardFrontTheme.dark,
);

const _kMockDrivingLicenseCardFront = CardFront(
  title: 'Rijbewijs',
  subtitle: 'Categorie AM, B, BE',
  logoImage: WalletAssets.logo_nl_driving_license,
  backgroundImage: WalletAssets.image_bg_nl_driving_license,
  theme: CardFrontTheme.light,
);

const _kMockDrivingLicenseRenewedCardFront = CardFront(
  title: 'Rijbewijs',
  subtitle: 'Categorie AM, B, C1, BE',
  logoImage: WalletAssets.logo_nl_driving_license,
  backgroundImage: WalletAssets.image_bg_nl_driving_license,
  theme: CardFrontTheme.light,
);

const _kMockHealthInsuranceCardFront = CardFront(
  title: 'European Health Insurance Card',
  subtitle: 'Zorgverzekeraar Z',
  logoImage: WalletAssets.logo_nl_health_insurance,
  backgroundImage: WalletAssets.image_bg_health_insurance,
  theme: CardFrontTheme.dark,
);

const _kMockVOGCardFront = CardFront(
  title: 'Verklaring Omtrent het Gedrag',
  info: 'Justis',
  logoImage: WalletAssets.logo_card_rijksoverheid,
  backgroundImage: WalletAssets.image_bg_diploma,
  theme: CardFrontTheme.dark,
);

// endregion

// region DataAttributes
const _kMockPidDataAttributes = [
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Voornamen',
    value: _kMockFirstNames,
    key: 'mock.firstNames',
    sourceCardId: _kPidId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Achternaam',
    value: _kMockLastName,
    key: 'mock.lastName',
    sourceCardId: _kPidId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Naam bij geboorte',
    value: 'Molenaar',
    key: 'mock.birthName',
    sourceCardId: _kPidId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Geslacht',
    value: _kMockGender,
    key: 'mock.gender',
    sourceCardId: _kPidId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Geboortedatum',
    value: _kMockBirthDate,
    key: 'mock.birthDate',
    sourceCardId: _kPidId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Ouder dan 18',
    value: 'Ja',
    key: 'mock.olderThan18',
    sourceCardId: _kPidId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Geboorteplaats',
    value: _kMockBirthPlace,
    key: 'mock.birthPlace',
    sourceCardId: _kPidId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Geboorteland',
    value: 'Nederland',
    key: 'mock.birthCountry',
    sourceCardId: _kPidId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Burgerservicenummer (BSN)',
    value: '******999',
    key: 'mock.citizenshipNumber',
    sourceCardId: _kPidId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Nationaliteit',
    value: 'Nederlands',
    key: 'mock.nationality',
    sourceCardId: _kPidId,
  ),
];

const _kMockAddressDataAttributes = [
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Straatnaam',
    value: 'Turfmarkt',
    key: 'mock.streetName',
    sourceCardId: _kAddressId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Huisnummer',
    value: '147',
    key: 'mock.houseNumber',
    sourceCardId: _kAddressId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Postcode',
    value: '2511 DP',
    key: 'mock.postalCode',
    sourceCardId: _kAddressId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Woonplaats',
    value: 'Den Haag',
    key: 'mock.city',
    sourceCardId: _kAddressId,
  ),
];

const _kMockDiplomaDataAttributes = [
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Onderwijsinstelling',
    value: 'Universiteit X',
    key: 'mock.university',
    sourceCardId: _kDiplomaId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Opleiding',
    value: 'WO Bachelor Bedrijfskunde',
    key: 'mock.education',
    sourceCardId: _kDiplomaId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Niveau',
    value: 'WO',
    key: 'mock.educationLevel',
    sourceCardId: _kDiplomaId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Type',
    value: 'Getuigschrift',
    sourceCardId: _kDiplomaId,
    key: 'mock.other',
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Uitgifte datum',
    value: '1 januari 2013',
    key: 'mock.issuanceDate',
    sourceCardId: _kDiplomaId,
  ),
];

const _kMockMasterDiplomaDataAttributes = [
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Onderwijsinstelling',
    value: 'Universiteit X',
    key: 'mock.university',
    sourceCardId: _kMasterDiplomaId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Opleiding',
    value: 'WO Master Bedrijfskunde',
    key: 'mock.education',
    sourceCardId: _kMasterDiplomaId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Niveau',
    value: 'WO',
    key: 'mock.educationLevel',
    sourceCardId: _kMasterDiplomaId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Type',
    value: 'Getuigschrift',
    sourceCardId: _kMasterDiplomaId,
    key: 'mock.other',
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Uitgifte datum',
    value: '1 januari 2015',
    key: 'mock.issuanceDate',
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
      value: WalletAssets.image_person_x,
      key: 'mock.profilePhoto',
      sourceCardId: _kDrivingLicenseId,
    ),
    const DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Voornamen',
      value: _kMockFirstNames,
      key: 'mock.firstNames',
      sourceCardId: _kDrivingLicenseId,
    ),
    const DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Naam',
      value: _kMockLastName,
      key: 'mock.lastName',
      sourceCardId: _kDrivingLicenseId,
    ),
    const DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Geboortedatum',
      value: _kMockBirthDate,
      key: 'mock.birthDate',
      sourceCardId: _kDrivingLicenseId,
    ),
    const DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Geboorteplaats',
      value: _kMockBirthPlace,
      key: 'mock.birthPlace',
      sourceCardId: _kDrivingLicenseId,
    ),
    const DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Afgiftedatum',
      value: '23-04-2018',
      key: 'mock.issuanceDate',
      sourceCardId: _kDrivingLicenseId,
    ),
    const DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Datum geldig tot',
      value: '23-04-2028',
      key: 'mock.expiryDate',
      sourceCardId: _kDrivingLicenseId,
    ),
    const DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Rijbewijsnummer',
      value: '99999999999',
      sourceCardId: _kDrivingLicenseId,
      key: 'mock.other',
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Rijbewijscategorieën',
      value: category,
      key: 'mock.drivingLicenseCategories',
      sourceCardId: _kDrivingLicenseId,
    ),
  ];
}

const _kMockHealthInsuranceDataAttributes = [
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Naam',
    value: _kMockFullName,
    key: 'mock.fullName',
    sourceCardId: _kHealthInsuranceId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Geslacht',
    value: _kMockGender,
    key: 'mock.gender',
    sourceCardId: _kHealthInsuranceId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Geboortedatum',
    value: _kMockBirthDate,
    key: 'mock.birthDate',
    sourceCardId: _kHealthInsuranceId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Klantnummer',
    value: '12345678',
    key: 'mock.healthIssuerClientId',
    sourceCardId: _kHealthInsuranceId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Kaartnummer',
    value: '9999999999',
    key: 'mock.documentNr',
    sourceCardId: _kHealthInsuranceId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'UZOVI',
    value: 'XXXX - 9999',
    key: 'mock.healthIssuerId',
    sourceCardId: _kHealthInsuranceId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Verloopdatum',
    value: '1 januari 2024',
    key: 'mock.healthInsuranceExpiryDate',
    sourceCardId: _kHealthInsuranceId,
  ),
];

const _kMockVOGDataAttributes = [
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Type',
    value: '1',
    key: 'mock.certificateOfConduct',
    sourceCardId: _kVOGId,
  ),
  DataAttribute(
    valueType: AttributeValueType.text,
    label: 'Datum geldig tot',
    value: '05-02-2023',
    key: 'mock.expiryDate',
    sourceCardId: _kVOGId,
  ),
];

// endregion

// region RequestedAttributes
const _kMockGovernmentOrganizationRequestedAttributes = [
  RequestedAttribute(
    label: 'BSN',
    key: 'mock.citizenshipNumber',
    valueType: AttributeValueType.text,
  ),
];

const _kMockHealthInsuranceRequestedAttributes = [
  RequestedAttribute(
    label: 'Voornamen',
    key: 'mock.firstNames',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Achternaam',
    key: 'mock.lastName',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Geboortedatum',
    key: 'mock.birthDate',
    valueType: AttributeValueType.text,
  ),
];
// endregion

// region Policies

const _kMockIssuancePolicy = Policy(
  storageDuration: Duration(days: 90),
  dataPurpose: 'Kaart uitgifte',
  dataIsShared: false,
  dataIsSignature: false,
  dataContainsSingleViewProfilePhoto: false,
  deletionCanBeRequested: true,
  privacyPolicyUrl: 'https://www.example.org',
);

// endregion
