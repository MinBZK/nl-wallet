part of 'mock_issuance_response_repository.dart';

final _kMockBirthDate = DateValue(DateTime(1997, 3, 10));
const _kMockBirthPlace = StringValue('Delft');
const _kMockFirstNames = StringValue('Willeke Liselotte');
const _kMockFullName = StringValue('Willeke De Bruijn');
const _kMockLastName = StringValue('De Bruijn');
const _kMockGender = StringValue('Vrouw');

// region WalletCards
const _kPidId = 'PID_1';
final _kMockPidWalletCard = WalletCard(
  id: _kPidId,
  issuerId: kRvigId,
  front: _kMockPidCardFront,
  attributes: _kMockPidDataAttributes,
  config: const CardConfig(),
);

const _kAddressId = 'ADDRESS_1';
final _kMockAddressWalletCard = WalletCard(
  id: _kAddressId,
  issuerId: kRvigId,
  front: _kMockAddressCardFront,
  attributes: _kMockAddressDataAttributes,
);

const _kDiplomaId = 'DIPLOMA_1';
final _kMockDiplomaWalletCard = WalletCard(
  id: _kDiplomaId,
  issuerId: kDuoId,
  front: _kMockDiplomaCardFront,
  attributes: _kMockDiplomaDataAttributes,
);

const _kMultiDiplomaId = 'MULTI_DIPLOMA';
const _kMasterDiplomaId = 'DIPLOMA_2';
final _kMockMasterDiplomaWalletCard = WalletCard(
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
final _kMockHealthInsuranceWalletCard = WalletCard(
  id: _kHealthInsuranceId,
  issuerId: kHealthInsuranceId,
  front: _kMockHealthInsuranceCardFront,
  attributes: _kMockHealthInsuranceDataAttributes,
);

const _kVOGId = 'VOG';
final _kMockVOGWalletCard = WalletCard(
  id: _kVOGId,
  issuerId: kJusticeId,
  front: _kMockVOGCardFront,
  attributes: _kMockVOGDataAttributes,
);

// endregion

// region CardFronts

final _kMockPidCardFront = CardFront(
  title: 'Persoonsgegevens'.untranslated,
  subtitle: 'Willeke'.untranslated,
  logoImage: WalletAssets.logo_card_rijksoverheid,
  holoImage: WalletAssets.svg_rijks_card_holo,
  backgroundImage: WalletAssets.svg_rijks_card_bg_light,
  theme: CardFrontTheme.light,
);

final _kMockAddressCardFront = CardFront(
  title: 'Woonadres'.untranslated,
  subtitle: "'s-Gravenhage".untranslated,
  logoImage: WalletAssets.logo_card_rijksoverheid,
  holoImage: WalletAssets.svg_rijks_card_holo,
  backgroundImage: WalletAssets.svg_rijks_card_bg_dark,
  theme: CardFrontTheme.dark,
);

final _kMockDiplomaCardFront = CardFront(
  title: 'BSc. Diploma'.untranslated,
  info: 'Dienst Uitvoerend Onderwijs'.untranslated,
  logoImage: WalletAssets.logo_card_rijksoverheid,
  backgroundImage: WalletAssets.image_bg_diploma,
  theme: CardFrontTheme.dark,
);

final _kMockMasterDiplomaCardFront = CardFront(
  title: 'MSc. Diploma'.untranslated,
  info: 'Dienst Uitvoerend Onderwijs'.untranslated,
  logoImage: WalletAssets.logo_card_rijksoverheid,
  backgroundImage: WalletAssets.image_bg_diploma,
  theme: CardFrontTheme.dark,
);

final _kMockDrivingLicenseCardFront = CardFront(
  title: 'Rijbewijs'.untranslated,
  subtitle: 'Categorie AM, B, BE'.untranslated,
  logoImage: WalletAssets.logo_nl_driving_license,
  backgroundImage: WalletAssets.image_bg_nl_driving_license,
  theme: CardFrontTheme.light,
);

final _kMockDrivingLicenseRenewedCardFront = CardFront(
  title: 'Rijbewijs'.untranslated,
  subtitle: 'Categorie AM, B, C1, BE'.untranslated,
  logoImage: WalletAssets.logo_nl_driving_license,
  backgroundImage: WalletAssets.image_bg_nl_driving_license,
  theme: CardFrontTheme.light,
);

final _kMockHealthInsuranceCardFront = CardFront(
  title: 'European Health Insurance Card'.untranslated,
  subtitle: 'Zorgverzekeraar Z'.untranslated,
  logoImage: WalletAssets.logo_nl_health_insurance,
  backgroundImage: WalletAssets.image_bg_health_insurance,
  theme: CardFrontTheme.dark,
);

final _kMockVOGCardFront = CardFront(
  title: 'Verklaring Omtrent het Gedrag'.untranslated,
  info: 'Justis'.untranslated,
  logoImage: WalletAssets.logo_card_rijksoverheid,
  backgroundImage: WalletAssets.image_bg_diploma,
  theme: CardFrontTheme.dark,
);

// endregion

// region DataAttributes
final _kMockPidDataAttributes = [
  DataAttribute.untranslated(
    label: 'Voornamen',
    value: _kMockFirstNames,
    key: 'mock.firstNames',
    sourceCardId: _kPidId,
  ),
  DataAttribute.untranslated(
    label: 'Achternaam',
    value: _kMockLastName,
    key: 'mock.lastName',
    sourceCardId: _kPidId,
  ),
  DataAttribute.untranslated(
    label: 'Naam bij geboorte',
    value: const StringValue('Molenaar'),
    key: 'mock.birthName',
    sourceCardId: _kPidId,
  ),
  DataAttribute.untranslated(
    label: 'Geslacht',
    value: _kMockGender,
    key: 'mock.gender',
    sourceCardId: _kPidId,
  ),
  DataAttribute.untranslated(
    label: 'Geboortedatum',
    value: _kMockBirthDate,
    key: 'mock.birthDate',
    sourceCardId: _kPidId,
  ),
  DataAttribute.untranslated(
    label: 'Ouder dan 18',
    value: const BooleanValue(true),
    key: 'mock.olderThan18',
    sourceCardId: _kPidId,
  ),
  DataAttribute.untranslated(
    label: 'Geboorteplaats',
    value: _kMockBirthPlace,
    key: 'mock.birthPlace',
    sourceCardId: _kPidId,
  ),
  DataAttribute.untranslated(
    label: 'Geboorteland',
    value: const StringValue('Nederland'),
    key: 'mock.birthCountry',
    sourceCardId: _kPidId,
  ),
  DataAttribute.untranslated(
    label: 'Burgerservicenummer (BSN)',
    value: const StringValue('******999'),
    key: 'mock.citizenshipNumber',
    sourceCardId: _kPidId,
  ),
  DataAttribute.untranslated(
    label: 'Nationaliteit',
    value: const StringValue('Nederlands'),
    key: 'mock.nationality',
    sourceCardId: _kPidId,
  ),
];

final _kMockAddressDataAttributes = [
  DataAttribute.untranslated(
    label: 'Straatnaam',
    value: const StringValue('Turfmarkt'),
    key: 'mock.streetName',
    sourceCardId: _kAddressId,
  ),
  DataAttribute.untranslated(
    label: 'Huisnummer',
    value: const StringValue('147'),
    key: 'mock.houseNumber',
    sourceCardId: _kAddressId,
  ),
  DataAttribute.untranslated(
    label: 'Postcode',
    value: const StringValue('2511 DP'),
    key: 'mock.postalCode',
    sourceCardId: _kAddressId,
  ),
  DataAttribute.untranslated(
    label: 'Woonplaats',
    value: const StringValue('Den Haag'),
    key: 'mock.city',
    sourceCardId: _kAddressId,
  ),
];

final _kMockDiplomaDataAttributes = [
  DataAttribute.untranslated(
    label: 'Onderwijsinstelling',
    value: const StringValue('Universiteit X'),
    key: 'mock.university',
    sourceCardId: _kDiplomaId,
  ),
  DataAttribute.untranslated(
    label: 'Opleiding',
    value: const StringValue('WO Bachelor Bedrijfskunde'),
    key: 'mock.education',
    sourceCardId: _kDiplomaId,
  ),
  DataAttribute.untranslated(
    label: 'Niveau',
    value: const StringValue('WO'),
    key: 'mock.educationLevel',
    sourceCardId: _kDiplomaId,
  ),
  DataAttribute.untranslated(
    label: 'Type',
    value: const StringValue('Getuigschrift'),
    sourceCardId: _kDiplomaId,
    key: 'mock.other',
  ),
  DataAttribute.untranslated(
    label: 'Uitgifte datum',
    value: DateValue(DateTime(2013, 1, 1)),
    key: 'mock.issuanceDate',
    sourceCardId: _kDiplomaId,
  ),
];

final _kMockMasterDiplomaDataAttributes = [
  DataAttribute.untranslated(
    label: 'Onderwijsinstelling',
    value: const StringValue('Universiteit X'),
    key: 'mock.university',
    sourceCardId: _kMasterDiplomaId,
  ),
  DataAttribute.untranslated(
    label: 'Opleiding',
    value: const StringValue('WO Master Bedrijfskunde'),
    key: 'mock.education',
    sourceCardId: _kMasterDiplomaId,
  ),
  DataAttribute.untranslated(
    label: 'Niveau',
    value: const StringValue('WO'),
    key: 'mock.educationLevel',
    sourceCardId: _kMasterDiplomaId,
  ),
  DataAttribute.untranslated(
    label: 'Type',
    value: const StringValue('Getuigschrift'),
    sourceCardId: _kMasterDiplomaId,
    key: 'mock.other',
  ),
  DataAttribute.untranslated(
    label: 'Uitgifte datum',
    value: DateValue(DateTime(2015, 1, 1)),
    key: 'mock.issuanceDate',
    sourceCardId: _kMasterDiplomaId,
  ),
];

final _kMockDrivingLicenseDataAttributes = _buildDrivingLicenseDataAttributes(category: 'AM, B, BE');
final _kMockDrivingLicenseRenewedDataAttributes = _buildDrivingLicenseDataAttributes(category: 'AM, B, C1, BE');

List<DataAttribute> _buildDrivingLicenseDataAttributes({required String category}) {
  return [
    DataAttribute.untranslated(
      label: 'Voornamen',
      value: _kMockFirstNames,
      key: 'mock.firstNames',
      sourceCardId: _kDrivingLicenseId,
    ),
    DataAttribute.untranslated(
      label: 'Naam',
      value: _kMockLastName,
      key: 'mock.lastName',
      sourceCardId: _kDrivingLicenseId,
    ),
    DataAttribute.untranslated(
      label: 'Geboortedatum',
      value: _kMockBirthDate,
      key: 'mock.birthDate',
      sourceCardId: _kDrivingLicenseId,
    ),
    DataAttribute.untranslated(
      label: 'Geboorteplaats',
      value: _kMockBirthPlace,
      key: 'mock.birthPlace',
      sourceCardId: _kDrivingLicenseId,
    ),
    DataAttribute.untranslated(
      label: 'Afgiftedatum',
      value: DateValue(DateTime(2018, 4, 23)),
      key: 'mock.issuanceDate',
      sourceCardId: _kDrivingLicenseId,
    ),
    DataAttribute.untranslated(
      label: 'Datum geldig tot',
      value: DateValue(DateTime(2028, 4, 23)),
      key: 'mock.expiryDate',
      sourceCardId: _kDrivingLicenseId,
    ),
    DataAttribute.untranslated(
      label: 'Rijbewijsnummer',
      value: const StringValue('99999999999'),
      sourceCardId: _kDrivingLicenseId,
      key: 'mock.other',
    ),
    DataAttribute.untranslated(
      label: 'Rijbewijscategorieën',
      value: StringValue(category),
      key: 'mock.drivingLicenseCategories',
      sourceCardId: _kDrivingLicenseId,
    ),
  ];
}

final _kMockHealthInsuranceDataAttributes = [
  DataAttribute.untranslated(
    label: 'Naam',
    value: _kMockFullName,
    key: 'mock.fullName',
    sourceCardId: _kHealthInsuranceId,
  ),
  DataAttribute.untranslated(
    label: 'Geslacht',
    value: _kMockGender,
    key: 'mock.gender',
    sourceCardId: _kHealthInsuranceId,
  ),
  DataAttribute.untranslated(
    label: 'Geboortedatum',
    value: _kMockBirthDate,
    key: 'mock.birthDate',
    sourceCardId: _kHealthInsuranceId,
  ),
  DataAttribute.untranslated(
    label: 'Klantnummer',
    value: const StringValue('12345678'),
    key: 'mock.healthIssuerClientId',
    sourceCardId: _kHealthInsuranceId,
  ),
  DataAttribute.untranslated(
    label: 'Kaartnummer',
    value: const StringValue('9999999999'),
    key: 'mock.documentNr',
    sourceCardId: _kHealthInsuranceId,
  ),
  DataAttribute.untranslated(
    label: 'UZOVI',
    value: const StringValue('XXXX - 9999'),
    key: 'mock.healthIssuerId',
    sourceCardId: _kHealthInsuranceId,
  ),
  DataAttribute.untranslated(
    label: 'Verloopdatum',
    value: DateValue(DateTime(2024, 1, 1)),
    key: 'mock.healthInsuranceExpiryDate',
    sourceCardId: _kHealthInsuranceId,
  ),
];

final _kMockVOGDataAttributes = [
  DataAttribute.untranslated(
    label: 'Type',
    value: const StringValue('1'),
    key: 'mock.certificateOfConduct',
    sourceCardId: _kVOGId,
  ),
  DataAttribute.untranslated(
    label: 'Datum geldig tot',
    value: DateValue(DateTime(2023, 2, 5)),
    key: 'mock.expiryDate',
    sourceCardId: _kVOGId,
  ),
];

// endregion

// region RequestedAttributes
final _kMockGovernmentOrganizationRequestedAttributes = [
  MissingAttribute.untranslated(
    label: 'BSN',
    key: 'mock.citizenshipNumber',
  ),
];

final _kMockHealthInsuranceRequestedAttributes = [
  MissingAttribute.untranslated(
    label: 'Voornamen',
    key: 'mock.firstNames',
  ),
  MissingAttribute.untranslated(
    label: 'Achternaam',
    key: 'mock.lastName',
  ),
  MissingAttribute.untranslated(
    label: 'Geboortedatum',
    key: 'mock.birthDate',
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
