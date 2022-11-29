part of 'mock_issuance_response_repository.dart';

const _kMockBirthDate = '10 maart 1997';
const _kMockBirthPlace = 'Delft';
const _kMockFirstNames = 'Willeke Liselotte';
const _kMockLastName = 'De Bruijn';

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

const _kMockPidDataAttributes = [
  DataAttribute(type: DataAttributeType.image, label: 'Pasfoto', value: 'assets/non-free/images/person_x.png'),
  DataAttribute(type: DataAttributeType.text, label: 'Voornamen', value: _kMockFirstNames),
  DataAttribute(type: DataAttributeType.text, label: 'Achternaam', value: _kMockLastName),
  DataAttribute(type: DataAttributeType.text, label: 'Geslachtsnaam', value: 'Molenaar'),
  DataAttribute(type: DataAttributeType.text, label: 'Geslacht', value: 'Vrouw'),
  DataAttribute(type: DataAttributeType.text, label: 'Geboortedatum', value: _kMockBirthDate),
  DataAttribute(type: DataAttributeType.text, label: 'Geboorteplaats', value: _kMockBirthPlace),
  DataAttribute(type: DataAttributeType.text, label: 'Geboorteland', value: 'Nederland'),
  DataAttribute(type: DataAttributeType.text, label: 'Burgerservicenummer (BSN)', value: '999999999'),
];

const _kMockDiplomaDataAttributes = [
  DataAttribute(type: DataAttributeType.text, label: 'Onderwijsinstelling', value: 'Universiteit X'),
  DataAttribute(type: DataAttributeType.text, label: 'Opleiding', value: 'WO Master Bedrijfskunde'),
  DataAttribute(type: DataAttributeType.text, label: 'Type', value: 'Getuigschrift'),
  DataAttribute(type: DataAttributeType.text, label: 'Uitgifte datum', value: '1 januari 2013'),
];

const _kMockPassportDataAttributes = [
  DataAttribute(type: DataAttributeType.image, label: 'Pasfoto', value: 'assets/non-free/images/person_x.png'),
  DataAttribute(type: DataAttributeType.text, label: 'Voornamen', value: _kMockFirstNames),
  DataAttribute(type: DataAttributeType.text, label: 'Naam', value: _kMockLastName),
  DataAttribute(type: DataAttributeType.text, label: 'Echtgenote van', value: 'Molenaar'),
  DataAttribute(type: DataAttributeType.text, label: 'Geboortedatum', value: _kMockBirthDate),
  DataAttribute(type: DataAttributeType.text, label: 'Geboorteplaats', value: _kMockBirthPlace),
  DataAttribute(type: DataAttributeType.text, label: 'Geslacht', value: 'Vrouw'),
  DataAttribute(type: DataAttributeType.text, label: 'Lengte', value: '1,75 m'),
  DataAttribute(type: DataAttributeType.text, label: 'Persoonsnummer', value: '9999999999'),
  DataAttribute(type: DataAttributeType.text, label: 'Documentnummer', value: 'SPECI2022'),
  DataAttribute(type: DataAttributeType.text, label: 'Datum verstrekking', value: '20 oktober 2014'),
  DataAttribute(type: DataAttributeType.text, label: 'Geldig tot', value: '20 OKT 2024'),
  DataAttribute(type: DataAttributeType.text, label: 'Type', value: 'P'),
  DataAttribute(type: DataAttributeType.text, label: 'Code', value: 'NL'),
];

const _kMockDrivingLicenseDataAttributes = [
  DataAttribute(type: DataAttributeType.image, label: 'Pasfoto', value: 'assets/non-free/images/person_x.png'),
  DataAttribute(type: DataAttributeType.text, label: 'Voornamen', value: _kMockFirstNames),
  DataAttribute(type: DataAttributeType.text, label: 'Naam', value: _kMockLastName),
  DataAttribute(type: DataAttributeType.text, label: 'Geboortedatum', value: _kMockBirthDate),
  DataAttribute(type: DataAttributeType.text, label: 'Geboorteplaats', value: _kMockBirthPlace),
  DataAttribute(type: DataAttributeType.text, label: 'Afgiftedatum', value: '23-04-2018'),
  DataAttribute(type: DataAttributeType.text, label: 'Datum geldig tot', value: '23-04-2028'),
  DataAttribute(type: DataAttributeType.text, label: 'Rijbewijsnummer', value: '99999999999'),
  DataAttribute(type: DataAttributeType.text, label: 'Rijbewijscategorieën', value: 'AM, B, C1, BE'),
];

const _kMockDiplomaRequestedAttributes = [
  DataAttribute(type: DataAttributeType.text, label: 'Voornamen', value: _kMockFirstNames),
  DataAttribute(type: DataAttributeType.text, label: 'Achternaam', value: _kMockLastName),
  DataAttribute(type: DataAttributeType.text, label: 'Geboortedatum', value: _kMockBirthDate),
];

const _kMockRequestedAttributes = [
  DataAttribute(type: DataAttributeType.text, label: 'Naam', value: _kMockLastName),
  DataAttribute(type: DataAttributeType.text, label: 'Geboortedatum', value: _kMockBirthDate),
];

const _kMockDrivingLicenseRequestedAttributes = [
  DataAttribute(type: DataAttributeType.text, label: 'Naam', value: _kMockLastName),
  DataAttribute(type: DataAttributeType.text, label: 'XXXX', value: 'XXXXX'),
];
