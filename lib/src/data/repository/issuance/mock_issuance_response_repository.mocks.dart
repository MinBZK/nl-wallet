part of 'mock_issuance_response_repository.dart';

const _kMockPidWalletCard = WalletCard(
  id: 'PID_1',
  front: _kMockPidCardFront,
  attributes: _kMockPidDataAttributes,
);

const _kMockPassportWalletCard = WalletCard(
  id: '1',
  front: _kMockPassportCardFront,
  attributes: _kMockAllDataAttributes,
);

const _kMockDrivingLicenseWalletCard = WalletCard(
  id: '2',
  front: _kMockDrivingLicenseCardFront,
  attributes: _kMockAllDataAttributes,
);

const _kMockPidCardFront = CardFront(
  title: 'Persoonsgegevens',
  info: 'W. de Bruijn',
  logoImage: 'assets/non-free/images/logo_rijksoverheid.png',
  backgroundImage: 'assets/images/bg_pid.png',
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
  DataAttribute(type: DataAttributeType.text, label: 'Voornamen', value: 'Willeke Liselotte'),
  DataAttribute(type: DataAttributeType.text, label: 'Achternaam', value: 'De Bruijn'),
  DataAttribute(type: DataAttributeType.text, label: 'Geslachtsnaam', value: 'Molenaar'),
  DataAttribute(type: DataAttributeType.text, label: 'Geslacht', value: 'Vrouw'),
  DataAttribute(type: DataAttributeType.text, label: 'Geboortedatum', value: '10 maart 1965'),
  DataAttribute(type: DataAttributeType.text, label: 'Geboorteplaats', value: 'Delft'),
  DataAttribute(type: DataAttributeType.text, label: 'Geboorteland', value: 'Nederland'),
  DataAttribute(type: DataAttributeType.text, label: 'Burgerservicenummer (BSN)', value: '999999999'),
];

const _kMockAllDataAttributes = [
  DataAttribute(type: DataAttributeType.image, label: 'Pasfoto', value: 'assets/non-free/images/person_x.png'),
  DataAttribute(type: DataAttributeType.text, label: 'Naam', value: 'De Bruijn'),
  DataAttribute(type: DataAttributeType.text, label: 'Echtgenote van', value: 'Molenaar'),
  DataAttribute(type: DataAttributeType.text, label: 'Voornamen', value: 'Willeke Liselotte'),
  DataAttribute(type: DataAttributeType.text, label: 'Geboortedatum', value: '10 maart 1965'),
  DataAttribute(type: DataAttributeType.text, label: 'Geboorteplaats', value: 'Delft'),
  DataAttribute(type: DataAttributeType.text, label: 'Geslacht', value: 'Vrouw'),
  DataAttribute(type: DataAttributeType.text, label: 'Lengte', value: '1,75 m'),
  DataAttribute(type: DataAttributeType.text, label: 'Persoonsnummer', value: '9999999999'),
  DataAttribute(type: DataAttributeType.text, label: 'Documentnummer', value: 'SPECI2022'),
  DataAttribute(type: DataAttributeType.text, label: 'Datum verstrekking', value: '20 oktober 2014'),
  DataAttribute(type: DataAttributeType.text, label: 'Geldig tot', value: '20 OKT 2024'),
  DataAttribute(type: DataAttributeType.text, label: 'Type', value: 'P'),
  DataAttribute(type: DataAttributeType.text, label: 'Code', value: 'NL'),
];

const _kMockRequestedAttributes = [
  DataAttribute(type: DataAttributeType.text, label: 'Naam', value: 'De Bruijn'),
  DataAttribute(type: DataAttributeType.text, label: 'Geboortedatum', value: '10 maart 1965'),
];
