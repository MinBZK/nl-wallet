part of 'mock_verification_request_repository.dart';

const _kJobApplicationId = 'JOB_APPLICATION';
const _kMarketplaceLoginId = 'MARKETPLACE_LOGIN';
const _kBarId = 'BAR';
const _kCarRental = 'CAR_RENTAL';
const _kFirstAid = 'FIRST_AID';

// region RequestedAttributes

const _kJobApplicationRequestedAttributes = [
  RequestedAttribute(
    name: 'Opleidingsnaam',
    type: AttributeType.education,
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    name: 'Onderwijsinstelling',
    type: AttributeType.university,
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    name: 'Niveau',
    type: AttributeType.educationLevel,
    valueType: AttributeValueType.text,
  ),
];

const _kMarketplaceLoginRequestedAttributes = [
  RequestedAttribute(
    name: 'Voornamen',
    type: AttributeType.firstNames,
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    name: 'Achternaam',
    type: AttributeType.lastName,
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    name: 'Postcode',
    type: AttributeType.postalCode,
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    name: 'Woonplaats',
    type: AttributeType.city,
    valueType: AttributeValueType.text,
  ),
];

const _kBarRequestedAttributes = [
  RequestedAttribute(
    name: 'Pasfoto',
    type: AttributeType.profilePhoto,
    valueType: AttributeValueType.image,
  ),
  RequestedAttribute(
    name: 'Ouder dan 18',
    type: AttributeType.olderThan18,
    valueType: AttributeValueType.text,
  ),
];

const _kCarRentalRequestedAttributes = [
  RequestedAttribute(
    name: 'Voornamen',
    type: AttributeType.firstNames,
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    name: 'Achternaam',
    type: AttributeType.lastName,
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    name: 'Geboortedatum',
    type: AttributeType.birthDate,
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    name: 'Rijbewijscategorieën',
    type: AttributeType.drivingLicenseCategories,
    valueType: AttributeValueType.text,
  ),
];

const _kFirstAidRequestedAttributes = [
  RequestedAttribute(
    name: 'Pasfoto',
    type: AttributeType.profilePhoto,
    valueType: AttributeValueType.image,
  ),
  RequestedAttribute(
    name: 'Voornamen',
    type: AttributeType.firstNames,
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    name: 'Achternaam',
    type: AttributeType.lastName,
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    name: 'Geslacht',
    type: AttributeType.gender,
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    name: 'Geboortedatum',
    type: AttributeType.birthDate,
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    name: 'Klantnummer',
    type: AttributeType.healthIssuerClientId,
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    name: 'Kaartnummer',
    type: AttributeType.documentNr,
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    name: 'UZOVI',
    type: AttributeType.healthIssuerId,
    valueType: AttributeValueType.text,
  ),
  RequestedAttribute(
    name: 'Verloopdatum',
    type: AttributeType.expiryDate,
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
  deletionCanBeRequested: true,
  privacyPolicyUrl: 'https://www.example.org',
);

const _kMockMarketPlacePolicy = Policy(
  storageDuration: Duration(days: 90),
  dataPurpose: 'Registreren',
  dataIsShared: false,
  dataIsSignature: false,
  deletionCanBeRequested: true,
  privacyPolicyUrl: 'https://www.example.org',
);

const _kMockBarPolicy = Policy(
  storageDuration: Duration(days: 0),
  dataPurpose: 'Leeftijd controle',
  dataIsShared: false,
  dataIsSignature: false,
  deletionCanBeRequested: true,
  privacyPolicyUrl: 'https://www.example.org',
);

const _kMockCarRentalPolicy = Policy(
  storageDuration: Duration(days: 90),
  dataPurpose: 'Rijvaardigheid',
  dataIsShared: false,
  dataIsSignature: false,
  deletionCanBeRequested: true,
  privacyPolicyUrl: 'https://www.example.org',
);

const _kMockFirstAidPolicy = Policy(
  storageDuration: Duration(days: 90),
  dataPurpose: 'Zorgverlening',
  dataIsShared: false,
  dataIsSignature: false,
  deletionCanBeRequested: true,
  privacyPolicyUrl: 'https://www.example.org',
);

// endregion
