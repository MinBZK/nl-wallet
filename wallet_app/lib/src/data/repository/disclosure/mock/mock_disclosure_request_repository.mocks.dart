part of 'mock_disclosure_request_repository.dart';

const _kJobApplicationId = 'JOB_APPLICATION';
const _kMarketplaceLoginId = 'MARKETPLACE_LOGIN';
const _kBarId = 'BAR';
const _kCarRental = 'CAR_RENTAL';
const _kFirstAid = 'FIRST_AID';
const _kParkingPermit = 'PARKING_PERMIT';
const _kOpenBankAccount = 'OPEN_BANK_ACCOUNT';
const _kProvideContractDetails = 'PROVIDE_CONTRACT_DETAILS';
const _kCreateMonkeyBikeAccount = 'CREATE_MB_ACCOUNT';
const _kPharmacy = 'PHARMACY';

// region RequestedAttributes

final _kFirstAndLastNameRequest = [
  MissingAttribute.untranslated(
    label: 'Voornamen',
    key: 'mock.firstNames',
  ),
  MissingAttribute.untranslated(
    label: 'Achternaam',
    key: 'mock.lastName',
  ),
];

final _kJobApplicationRequestedAttributes = [
  MissingAttribute.untranslated(
    label: 'Opleidingsnaam',
    key: 'mock.education',
  ),
  MissingAttribute.untranslated(
    label: 'Onderwijsinstelling',
    key: 'mock.university',
  ),
  MissingAttribute.untranslated(
    label: 'Niveau',
    key: 'mock.educationLevel',
  ),
];

final _kMarketplaceLoginRequestedAttributes = [
  ..._kFirstAndLastNameRequest,
  MissingAttribute.untranslated(
    label: 'Geboortedatum',
    key: 'mock.birthDate',
  ),
  MissingAttribute.untranslated(
    label: 'Postcode',
    key: 'mock.postalCode',
  ),
];

final _kBarRequestedAttributes = [
  MissingAttribute.untranslated(
    label: 'Pasfoto',
    key: 'mock.profilePhoto',
  ),
  MissingAttribute.untranslated(
    label: 'Ouder dan 18',
    key: 'mock.olderThan18',
  ),
];

final _kCarRentalRequestedAttributes = [
  ..._kFirstAndLastNameRequest,
  MissingAttribute.untranslated(
    label: 'Geboortedatum',
    key: 'mock.birthDate',
  ),
  MissingAttribute.untranslated(
    label: 'Rijbewijscategorieën',
    key: 'mock.drivingLicenseCategories',
  ),
];

final _kFirstAidRequestedAttributes = [
  MissingAttribute.untranslated(
    label: 'Pasfoto',
    key: 'mock.profilePhoto',
  ),
  ..._kFirstAndLastNameRequest,
  MissingAttribute.untranslated(
    label: 'Geslacht',
    key: 'mock.gender',
  ),
  MissingAttribute.untranslated(
    label: 'Geboortedatum',
    key: 'mock.birthDate',
  ),
  MissingAttribute.untranslated(
    label: 'Klantnummer',
    key: 'mock.healthIssuerClientId',
  ),
  MissingAttribute.untranslated(
    label: 'Kaartnummer',
    key: 'mock.documentNr',
  ),
  MissingAttribute.untranslated(
    label: 'UZOVI',
    key: 'mock.healthIssuerId',
  ),
  MissingAttribute.untranslated(
    label: 'Verloopdatum',
    key: 'mock.healthInsuranceExpiryDate',
  ),
];

final _kPharmacyRequestedAttributes = [
  ..._kFirstAndLastNameRequest,
  MissingAttribute.untranslated(
    label: 'Huisnummer',
    key: 'mock.houseNumber',
  ),
  MissingAttribute.untranslated(
    label: 'Postcode',
    key: 'mock.postalCode',
  ),
];

final _kParkingPermitRequestedAttributes = [
  MissingAttribute.untranslated(
    label: 'Achternaam',
    key: 'mock.lastName',
  ),
  MissingAttribute.untranslated(
    label: 'Postcode',
    key: 'mock.postalCode',
  ),
  MissingAttribute.untranslated(
    label: 'Huisnummer',
    key: 'mock.houseNumber',
  ),
];

final _kOpenBankAccountRequestedAttributes = [
  ..._kFirstAndLastNameRequest,
  MissingAttribute.untranslated(
    label: 'Geboortedatum',
    key: 'mock.birthDate',
  ),
  MissingAttribute.untranslated(
    label: 'Nationaliteit',
    key: 'mock.nationality',
  ),
  MissingAttribute.untranslated(
    label: 'Burgerservicenummer',
    key: 'mock.citizenshipNumber',
  ),
  MissingAttribute.untranslated(
    label: 'Straatnaam',
    key: 'mock.streetName',
  ),
  MissingAttribute.untranslated(
    label: 'Huisnummer',
    key: 'mock.houseNumber',
  ),
  MissingAttribute.untranslated(
    label: 'Postcode',
    key: 'mock.postalCode',
  ),
];

final _kProvideContractDetailsRequestedAttributes = [
  ..._kFirstAndLastNameRequest,
  MissingAttribute.untranslated(
    label: 'Geboortedatum',
    key: 'mock.birthDate',
  ),
];

final _kCreateMbAccountRequestedAttributes = [
  ..._kFirstAndLastNameRequest,
  MissingAttribute.untranslated(
    label: 'Geboortedatum',
    key: 'mock.birthDate',
  ),
  MissingAttribute.untranslated(
    label: 'Nationaliteit',
    key: 'mock.nationality',
  ),
  MissingAttribute.untranslated(
    label: 'Straatnaam',
    key: 'mock.streetName',
  ),
  MissingAttribute.untranslated(
    label: 'Huisnummer',
    key: 'mock.houseNumber',
  ),
  MissingAttribute.untranslated(
    label: 'Postcode',
    key: 'mock.postalCode',
  ),
  MissingAttribute.untranslated(
    label: 'Woonplaats',
    key: 'mock.city',
  ),
];

// endregion

// region InteractionPolicies

const _kEmployerPolicy = Policy(
  storageDuration: Duration(days: 3 * 30),
  dataPurpose: 'Gegevens controle',
  dataIsShared: false,
  dataIsSignature: false,
  dataContainsSingleViewProfilePhoto: false,
  deletionCanBeRequested: true,
  privacyPolicyUrl: 'https://www.example.org',
);

const _kMockMarketPlacePolicy = Policy(
  storageDuration: Duration(days: 90),
  dataPurpose: 'Registreren',
  dataIsShared: false,
  dataIsSignature: false,
  dataContainsSingleViewProfilePhoto: false,
  deletionCanBeRequested: true,
  privacyPolicyUrl: 'https://www.example.org',
);

const _kMockBarPolicy = Policy(
  storageDuration: Duration(days: 0),
  dataPurpose: 'Leeftijd controle',
  dataIsShared: false,
  dataIsSignature: false,
  dataContainsSingleViewProfilePhoto: true,
  deletionCanBeRequested: false,
  privacyPolicyUrl: 'https://www.example.org',
);

const _kMockCarRentalPolicy = Policy(
  storageDuration: Duration(days: 90),
  dataPurpose: 'Rijvaardigheid',
  dataIsShared: false,
  dataIsSignature: false,
  dataContainsSingleViewProfilePhoto: false,
  deletionCanBeRequested: true,
  privacyPolicyUrl: 'https://www.example.org',
);

const _kMockFirstAidPolicy = Policy(
  storageDuration: Duration(days: 365),
  dataPurpose: 'Zorgverlening',
  dataIsShared: true,
  dataIsSignature: false,
  dataContainsSingleViewProfilePhoto: false,
  deletionCanBeRequested: true,
  privacyPolicyUrl: 'https://www.example.org',
);

const _kMockMunicipalityGenericPolicy = Policy(
  storageDuration: Duration(days: 90),
  dataPurpose: 'Gegevens dienen uitsluitend als bewijs',
  dataIsShared: false,
  dataIsSignature: false,
  dataContainsSingleViewProfilePhoto: false,
  deletionCanBeRequested: true,
  privacyPolicyUrl: 'https://www.example.org',
);

const _kMockBankPolicy = Policy(
  storageDuration: Duration(days: 90),
  dataPurpose: 'Gegevens dienen uitsluitend als bewijs',
  dataIsShared: false,
  dataIsSignature: false,
  dataContainsSingleViewProfilePhoto: false,
  deletionCanBeRequested: true,
  privacyPolicyUrl: 'https://www.example.org',
);

const _kMockHousingCorpPolicy = Policy(
  storageDuration: Duration(days: 90),
  dataPurpose: 'Gegevens dienen uitsluitend als bewijs',
  dataIsShared: false,
  dataIsSignature: false,
  dataContainsSingleViewProfilePhoto: false,
  deletionCanBeRequested: true,
  privacyPolicyUrl: 'https://www.example.org',
);

const _kMonkeyBikePolicy = Policy(
  storageDuration: Duration(days: 90),
  dataPurpose: 'Gegevens worden ook gebruikt voor andere doelen',
  dataPurposeDescription: 'De gegevens kunnen worden gebruikt voor marketing en personalisatie.',
  dataIsShared: true,
  dataIsSignature: false,
  dataContainsSingleViewProfilePhoto: false,
  deletionCanBeRequested: true,
  privacyPolicyUrl: 'https://www.example.org',
);

// endregion
