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

// region RequestedAttributes

const _kFirstAndLastNameRequest = [
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
];

const _kJobApplicationRequestedAttributes = [
  RequestedAttribute(
    label: 'Opleidingsnaam',
    key: 'mock.education',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Onderwijsinstelling',
    key: 'mock.university',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Niveau',
    key: 'mock.educationLevel',
    valueType: AttributeValueType.text,
  ),
];

const _kMarketplaceLoginRequestedAttributes = [
  ..._kFirstAndLastNameRequest,
  RequestedAttribute(
    label: 'Geboortedatum',
    key: 'mock.birthDate',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Postcode',
    key: 'mock.postalCode',
    valueType: AttributeValueType.text,
  ),
];

const _kBarRequestedAttributes = [
  RequestedAttribute(
    label: 'Pasfoto',
    key: 'mock.profilePhoto',
    valueType: AttributeValueType.image,
  ),
  RequestedAttribute(
    label: 'Ouder dan 18',
    key: 'mock.olderThan18',
    valueType: AttributeValueType.text,
  ),
];

const _kCarRentalRequestedAttributes = [
  ..._kFirstAndLastNameRequest,
  RequestedAttribute(
    label: 'Geboortedatum',
    key: 'mock.birthDate',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Rijbewijscategorieën',
    key: 'mock.drivingLicenseCategories',
    valueType: AttributeValueType.text,
  ),
];

const _kFirstAidRequestedAttributes = [
  RequestedAttribute(
    label: 'Pasfoto',
    key: 'mock.profilePhoto',
    valueType: AttributeValueType.image,
  ),
  ..._kFirstAndLastNameRequest,
  RequestedAttribute(
    label: 'Geslacht',
    key: 'mock.gender',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Geboortedatum',
    key: 'mock.birthDate',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Klantnummer',
    key: 'mock.healthIssuerClientId',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Kaartnummer',
    key: 'mock.documentNr',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'UZOVI',
    key: 'mock.healthIssuerId',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Verloopdatum',
    key: 'mock.healthInsuranceExpiryDate',
    valueType: AttributeValueType.text,
  ),
];

const _kParkingPermitRequestedAttributes = [
  RequestedAttribute(
    label: 'Achternaam',
    key: 'mock.lastName',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Postcode',
    key: 'mock.postalCode',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Huisnummer',
    key: 'mock.houseNumber',
    valueType: AttributeValueType.text,
  ),
];

const _kOpenBankAccountRequestedAttributes = [
  ..._kFirstAndLastNameRequest,
  RequestedAttribute(
    label: 'Geboortedatum',
    key: 'mock.birthDate',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Nationaliteit',
    key: 'mock.nationality',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Burgerservicenummer',
    key: 'mock.citizenshipNumber',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Straatnaam',
    key: 'mock.streetName',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Huisnummer',
    key: 'mock.houseNumber',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Postcode',
    key: 'mock.postalCode',
    valueType: AttributeValueType.text,
  ),
];

const _kProvideContractDetailsRequestedAttributes = [
  ..._kFirstAndLastNameRequest,
  RequestedAttribute(
    label: 'Geboortedatum',
    key: 'mock.birthDate',
    valueType: AttributeValueType.text,
  ),
];

const _kCreateMbAccountRequestedAttributes = [
  ..._kFirstAndLastNameRequest,
  RequestedAttribute(
    label: 'Geboortedatum',
    key: 'mock.birthDate',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Nationaliteit',
    key: 'mock.nationality',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Straatnaam',
    key: 'mock.streetName',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Huisnummer',
    key: 'mock.houseNumber',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Postcode',
    key: 'mock.postalCode',
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    label: 'Woonplaats',
    key: 'mock.city',
    valueType: AttributeValueType.text,
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
  storageDuration: Duration(days: 90),
  dataPurpose: 'Zorgverlening',
  dataIsShared: false,
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
