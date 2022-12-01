part of 'mock_issuance_response_repository.dart';

const _kMockBirthDate = '10 maart 1997';
const _kMockBirthPlace = 'Delft';
const _kMockFirstNames = 'Willeke Liselotte';
const _kMockLastName = 'De Bruijn';

// region WalletCards
const _kMockPidWalletCard = WalletCard(
  id: 'PID_1',
  front: _kMockPidCardFront,
  attributes: _kMockPidDataAttributes,
);

const _kMockDiplomaWalletCard = WalletCard(
  id: 'DIPLOMA_1',
  front: _kMockDiplomaCardFront,
  attributes: _kMockDiplomaDataAttributes,
);

const _kMockPassportWalletCard = WalletCard(
  id: 'PASSPORT',
  front: _kMockPassportCardFront,
  attributes: _kMockPassportDataAttributes,
);

const _kMockDrivingLicenseWalletCard = WalletCard(
  id: 'DRIVING_LICENSE',
  front: _kMockDrivingLicenseCardFront,
  attributes: _kMockDrivingLicenseDataAttributes,
);

const _kMockVOGWalletCard = WalletCard(
  id: 'VOG',
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
  title: 'Diploma',
  info: 'Dienst Uitvoerend Onderwijs',
  logoImage: 'assets/non-free/images/logo_card_rijksoverheid.png',
  backgroundImage: 'assets/images/bg_diploma.png',
  theme: CardFrontTheme.dark,
);

const _kMockPassportCardFront = CardFront(
  title: 'Paspoort',
  info: 'Koninkrijk der Nederlanden',
  logoImage: 'assets/non-free/images/logo_nl_passport.png',
  backgroundImage: 'assets/images/bg_nl_passport.png',
  theme: CardFrontTheme.light,
);

const _kMockDrivingLicenseCardFront = CardFront(
  title: 'Rijbewijs',
  info: 'Categorie AM, B, C1, BE',
  logoImage: 'assets/non-free/images/logo_nl_driving_license.png',
  backgroundImage: 'assets/images/bg_nl_driving_license.png',
  theme: CardFrontTheme.light,
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
    valueType: DataAttributeValueType.image,
    label: 'Pasfoto',
    value: 'assets/non-free/images/person_x.png',
    type: DataAttributeType.profilePhoto,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Voornamen',
    value: _kMockFirstNames,
    type: DataAttributeType.firstName,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Achternaam',
    value: _kMockLastName,
    type: DataAttributeType.lastName,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Geslachtsnaam',
    value: 'Molenaar',
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Geslacht',
    value: 'Vrouw',
    type: DataAttributeType.gender,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Geboortedatum',
    value: _kMockBirthDate,
    type: DataAttributeType.birthDate,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Ouder dan 18',
    value: 'Ja',
    type: DataAttributeType.olderThan18,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Geboorteplaats',
    value: _kMockBirthPlace,
    type: DataAttributeType.birthPlace,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Geboorteland',
    value: 'Nederland',
    type: DataAttributeType.birthCountry,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Burgerservicenummer (BSN)',
    value: '999999999',
    type: DataAttributeType.citizenshipNumber,
  ),
];

const _kMockDiplomaDataAttributes = [
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Onderwijsinstelling',
    value: 'Universiteit X',
    type: DataAttributeType.university,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Opleiding',
    value: 'WO Master Bedrijfskunde',
    type: DataAttributeType.education,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Niveau',
    value: 'WO',
    type: DataAttributeType.educationLevel,
  ),
  DataAttribute(valueType: DataAttributeValueType.text, label: 'Type', value: 'Getuigschrift'),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Uitgifte datum',
    value: '1 januari 2013',
    type: DataAttributeType.issuanceDate,
  ),
];

const _kMockPassportDataAttributes = [
  DataAttribute(
    valueType: DataAttributeValueType.image,
    label: 'Pasfoto',
    value: 'assets/non-free/images/person_x.png',
    type: DataAttributeType.profilePhoto,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Voornamen',
    value: _kMockFirstNames,
    type: DataAttributeType.firstName,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Naam',
    value: _kMockLastName,
    type: DataAttributeType.lastName,
  ),
  DataAttribute(valueType: DataAttributeValueType.text, label: 'Echtgenote van', value: 'Molenaar'),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Geboortedatum',
    value: _kMockBirthDate,
    type: DataAttributeType.birthDate,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Geboorteplaats',
    value: _kMockBirthPlace,
    type: DataAttributeType.birthPlace,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Geslacht',
    value: 'Vrouw',
    type: DataAttributeType.gender,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Lengte',
    value: '1,75 m',
    type: DataAttributeType.height,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Persoonsnummer',
    value: '9999999999',
    type: DataAttributeType.citizenshipNumber,
  ),
  DataAttribute(valueType: DataAttributeValueType.text, label: 'Documentnummer', value: 'SPECI2022'),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Datum verstrekking',
    value: '20 oktober 2014',
    type: DataAttributeType.issuanceDate,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Geldig tot',
    value: '20 OKT 2024',
    type: DataAttributeType.expiryDate,
  ),
  DataAttribute(valueType: DataAttributeValueType.text, label: 'Type', value: 'P'),
  DataAttribute(valueType: DataAttributeValueType.text, label: 'Code', value: 'NL'),
];

const _kMockDrivingLicenseDataAttributes = [
  DataAttribute(
    valueType: DataAttributeValueType.image,
    label: 'Pasfoto',
    value: 'assets/non-free/images/person_x.png',
    type: DataAttributeType.profilePhoto,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Voornamen',
    value: _kMockFirstNames,
    type: DataAttributeType.firstName,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Naam',
    value: _kMockLastName,
    type: DataAttributeType.lastName,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Geboortedatum',
    value: _kMockBirthDate,
    type: DataAttributeType.birthDate,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Geboorteplaats',
    value: _kMockBirthPlace,
    type: DataAttributeType.birthPlace,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Afgiftedatum',
    value: '23-04-2018',
    type: DataAttributeType.issuanceDate,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Datum geldig tot',
    value: '23-04-2028',
    type: DataAttributeType.expiryDate,
  ),
  DataAttribute(valueType: DataAttributeValueType.text, label: 'Rijbewijsnummer', value: '99999999999'),
  DataAttribute(valueType: DataAttributeValueType.text, label: 'Rijbewijscategorieën', value: 'AM, B, C1, BE'),
];

const _kMockVOGDataAttributes = [
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Type',
    value: '1',
    type: DataAttributeType.certificateOfConduct,
  ),
  DataAttribute(
    valueType: DataAttributeValueType.text,
    label: 'Datum geldig tot',
    value: '05-02-2023',
    type: DataAttributeType.expiryDate,
  ),
];

// endregion

// region RequestedAttributes
const _kMockDiplomaRequestedAttributes = [
  RequestedAttribute(name: 'Voornamen', type: DataAttributeType.firstName, valueType: DataAttributeValueType.text),
  RequestedAttribute(name: 'Achternaam', type: DataAttributeType.lastName, valueType: DataAttributeValueType.text),
  RequestedAttribute(name: 'Geboortedatum', type: DataAttributeType.birthDate, valueType: DataAttributeValueType.text),
];

const _kMockGenericRequestedAttributes = [
  RequestedAttribute(name: 'BSN', type: DataAttributeType.citizenshipNumber, valueType: DataAttributeValueType.text),
];
// endregion
