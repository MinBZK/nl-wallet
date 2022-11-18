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

const _kMockLicenseWalletCard = WalletCard(
  id: '2',
  front: _kMockLicenseCardFront,
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

const _kMockLicenseCardFront = CardFront(
  title: 'Rijbewijs',
  info: 'Categorie AM, B, C1, BE',
  logoImage: 'assets/non-free/images/logo_nl_driving_license.png',
  backgroundImage: 'assets/images/bg_nl_driving_license.png',
  theme: CardFrontTheme.light,
);

const _kMockPidDataAttributes = [
  DataAttribute(type: 'Image', value: 'assets/non-free/images/person_x.png'),
  DataAttribute(type: 'Voornamen', value: 'Willeke Liselotte'),
  DataAttribute(type: 'Achternaam', value: 'De Bruijn'),
  DataAttribute(type: 'Geslachtsnaam', value: 'Molenaar'),
  DataAttribute(type: 'Geslacht', value: 'Vrouw'),
  DataAttribute(type: 'Geboortedatum', value: '10 maart 1965'),
  DataAttribute(type: 'Geboorteplaats', value: 'Delft'),
  DataAttribute(type: 'Geboorteland', value: 'Nederland'),
  DataAttribute(type: 'Burgerservicenummer (BSN)', value: '999999999'),
];

const _kMockAllDataAttributes = [
  DataAttribute(type: 'Image', value: 'assets/non-free/images/person_x.png'),
  DataAttribute(type: 'Naam', value: 'De Bruijn'),
  DataAttribute(type: 'Echtgenote van', value: 'Molenaar'),
  DataAttribute(type: 'Voornamen', value: 'Willeke Liselotte'),
  DataAttribute(type: 'Geboortedatum', value: '10 maart 1965'),
  DataAttribute(type: 'Geboorteplaats', value: 'Delft'),
  DataAttribute(type: 'Geslacht', value: 'Vrouw'),
  DataAttribute(type: 'Lengte', value: '1,75 m'),
  DataAttribute(type: 'Persoonsnummer', value: '9999999999'),
  DataAttribute(type: 'Documentnummer', value: 'SPECI2022'),
  DataAttribute(type: 'Datum verstrekking', value: '20 oktober 2014'),
  DataAttribute(type: 'Geldig tot', value: '20 OKT 2024'),
  DataAttribute(type: 'Type', value: 'P'),
  DataAttribute(type: 'Code', value: 'NL'),
];

const _kMockRequestedAttributes = [
  DataAttribute(type: 'Naam', value: 'De Bruijn'),
  DataAttribute(type: 'Geboortedatum', value: '10 maart 1965'),
];
