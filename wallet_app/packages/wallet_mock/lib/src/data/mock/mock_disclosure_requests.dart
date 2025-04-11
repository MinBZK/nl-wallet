import 'package:wallet_core/core.dart';

import '../model/disclosure_request.dart';
import '../model/requested_attribute.dart';
import 'mock_organizations.dart';

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
const _kAmsterdamLoginId = 'AMSTERDAM_LOGIN';

const _kMockBirthDateKey = 'mock_birthDate';
const _kMockStreetNameKey = 'mock_streetName';
const _kMockPostalCodeKey = 'mock_postalCode';
const _kMockHouseNumberKey = 'mock_houseNumber';
const _kMockUrl = 'https://www.example.org';

final List<DisclosureRequest> kDisclosureRequests = [
  DisclosureRequest(
    id: _kJobApplicationId,
    relyingParty: kOrganizations[kEmployerId]!,
    requestedAttributes: _kJobApplicationRequestedAttributes,
    purpose: 'Sollicitatie',
    policy: _kEmployerPolicy,
  ),
  DisclosureRequest(
    id: _kMarketplaceLoginId,
    relyingParty: kOrganizations[kMarketplaceId]!,
    requestedAttributes: _kMarketplaceLoginRequestedAttributes,
    purpose: 'Account aanmaken',
    policy: _kMockMarketPlacePolicy,
  ),
  DisclosureRequest(
    id: _kBarId,
    relyingParty: kOrganizations[kBarId]!,
    requestedAttributes: _kBarRequestedAttributes,
    purpose: 'Leeftijd controle',
    policy: _kMockBarPolicy,
  ),
  DisclosureRequest(
    id: _kCarRental,
    relyingParty: kOrganizations[kCarRentalId]!,
    requestedAttributes: _kCarRentalRequestedAttributes,
    purpose: 'Gegevens & Rijbewijs controle',
    policy: _kMockCarRentalPolicy,
  ),
  DisclosureRequest(
    id: _kFirstAid,
    relyingParty: kOrganizations[kFirstAidId]!,
    requestedAttributes: _kFirstAidRequestedAttributes,
    purpose: 'Gegevens & Verzekering controle',
    policy: _kMockFirstAidPolicy,
  ),
  DisclosureRequest(
    id: _kParkingPermit,
    relyingParty: kOrganizations[kMunicipalityTheHagueId]!,
    requestedAttributes: _kParkingPermitRequestedAttributes,
    purpose: 'Parkeervergunning',
    policy: _kMockMunicipalityGenericPolicy,
  ),
  DisclosureRequest(
    id: _kOpenBankAccount,
    relyingParty: kOrganizations[kBankId]!,
    requestedAttributes: _kOpenBankAccountRequestedAttributes,
    purpose: 'Rekening openen',
    policy: _kMockBankPolicy,
  ),
  DisclosureRequest(
    id: _kProvideContractDetails,
    relyingParty: kOrganizations[kHousingCorpId]!,
    requestedAttributes: _kProvideContractDetailsRequestedAttributes,
    purpose: 'Identificeren',
    policy: _kMockHousingCorpPolicy,
  ),
  DisclosureRequest(
    id: _kCreateMonkeyBikeAccount,
    relyingParty: kOrganizations[kMonkeyBikeId]!,
    requestedAttributes: _kCreateMbAccountRequestedAttributes,
    purpose: 'Account aanmaken',
    policy: _kMonkeyBikePolicy,
  ),
  DisclosureRequest(
    id: _kPharmacy,
    relyingParty: kOrganizations[kPharmacyId]!,
    requestedAttributes: _kPharmacyRequestedAttributes,
    purpose: 'Herhaalrecept',
    policy: _kMockFirstAidPolicy,
  ),
  DisclosureRequest(
    id: _kAmsterdamLoginId,
    relyingParty: kOrganizations[kMunicipalityAmsterdamId]!,
    requestedAttributes: _kCitizenshipNumberRequest,
    purpose: 'Inloggen',
    policy: _kMunicipalityAmsterdamPolicy,
  ),
];

// region RequestedAttributes
final _kFirstAndLastNameRequest = [
  RequestedAttribute(
    label: 'Voornamen',
    key: 'mock_firstNames',
  ),
  RequestedAttribute(
    label: 'Achternaam',
    key: 'mock_lastName',
  ),
];

final _kCitizenshipNumberRequest = [
  RequestedAttribute(
    label: 'Burger­service­nummer',
    key: 'mock_citizenshipNumber',
  ),
];

final _kJobApplicationRequestedAttributes = [
  RequestedAttribute(
    label: 'Opleidingsnaam',
    key: 'mock_education',
  ),
  RequestedAttribute(
    label: 'Onderwijsinstelling',
    key: 'mock_university',
  ),
  RequestedAttribute(
    label: 'Niveau',
    key: 'mock_educationLevel',
  ),
];

final _kMarketplaceLoginRequestedAttributes = [
  ..._kFirstAndLastNameRequest,
  RequestedAttribute(
    label: 'Geboortedatum',
    key: _kMockBirthDateKey,
  ),
  RequestedAttribute(
    label: 'Straatnaam',
    key: _kMockStreetNameKey,
  ),
  RequestedAttribute(
    label: 'Huisnummer',
    key: _kMockHouseNumberKey,
  ),
  RequestedAttribute(
    label: 'Postcode',
    key: _kMockPostalCodeKey,
  ),
];

final _kBarRequestedAttributes = [
  RequestedAttribute(
    label: 'Pasfoto',
    key: 'mock_profilePhoto',
  ),
  RequestedAttribute(
    label: 'Ouder dan 18',
    key: 'mock_olderThan18',
  ),
];

final _kCarRentalRequestedAttributes = [
  ..._kFirstAndLastNameRequest,
  RequestedAttribute(
    label: 'Geboortedatum',
    key: _kMockBirthDateKey,
  ),
  RequestedAttribute(
    label: 'Rijbewijscategorieën',
    key: 'mock_drivingLicenseCategories',
  ),
];

final _kFirstAidRequestedAttributes = [
  RequestedAttribute(
    label: 'Pasfoto',
    key: 'mock_profilePhoto',
  ),
  ..._kFirstAndLastNameRequest,
  RequestedAttribute(
    label: 'Geslacht',
    key: 'mock_gender',
  ),
  RequestedAttribute(
    label: 'Geboortedatum',
    key: _kMockBirthDateKey,
  ),
  RequestedAttribute(
    label: 'Klantnummer',
    key: 'mock_healthIssuerClientId',
  ),
  RequestedAttribute(
    label: 'Kaartnummer',
    key: 'mock_documentNr',
  ),
  RequestedAttribute(
    label: 'UZOVI',
    key: 'mock_healthIssuerId',
  ),
  RequestedAttribute(
    label: 'Verloopdatum',
    key: 'mock_healthInsuranceExpiryDate',
  ),
];

final _kPharmacyRequestedAttributes = [
  ..._kFirstAndLastNameRequest,
  RequestedAttribute(
    label: 'Huisnummer',
    key: _kMockHouseNumberKey,
  ),
  RequestedAttribute(
    label: 'Postcode',
    key: _kMockPostalCodeKey,
  ),
];

final _kParkingPermitRequestedAttributes = [
  RequestedAttribute(
    label: 'Achternaam',
    key: 'mock_lastName',
  ),
  RequestedAttribute(
    label: 'Postcode',
    key: _kMockPostalCodeKey,
  ),
  RequestedAttribute(
    label: 'Huisnummer',
    key: _kMockHouseNumberKey,
  ),
];

final _kOpenBankAccountRequestedAttributes = [
  ..._kFirstAndLastNameRequest,
  RequestedAttribute(
    label: 'Geboortedatum',
    key: _kMockBirthDateKey,
  ),
  ..._kCitizenshipNumberRequest,
  RequestedAttribute(
    label: 'Straatnaam',
    key: _kMockStreetNameKey,
  ),
  RequestedAttribute(
    label: 'Huisnummer',
    key: _kMockHouseNumberKey,
  ),
  RequestedAttribute(
    label: 'Postcode',
    key: _kMockPostalCodeKey,
  ),
];

final _kProvideContractDetailsRequestedAttributes = [
  ..._kFirstAndLastNameRequest,
  RequestedAttribute(
    label: 'Geboortedatum',
    key: _kMockBirthDateKey,
  ),
];

final _kCreateMbAccountRequestedAttributes = [
  ..._kFirstAndLastNameRequest,
  RequestedAttribute(
    label: 'Geboortedatum',
    key: _kMockBirthDateKey,
  ),
  RequestedAttribute(
    label: 'Straatnaam',
    key: _kMockStreetNameKey,
  ),
  RequestedAttribute(
    label: 'Huisnummer',
    key: _kMockHouseNumberKey,
  ),
  RequestedAttribute(
    label: 'Postcode',
    key: _kMockPostalCodeKey,
  ),
  RequestedAttribute(
    label: 'Woonplaats',
    key: 'mock_city',
  ),
];

// endregion

// region InteractionPolicies

final _kEmployerPolicy = RequestPolicy(
  dataStorageDurationInMinutes: BigInt.from(60 * 24 * 90),
// dataPurpose: 'Gegevens controle',
  dataSharedWithThirdParties: false,
  dataDeletionPossible: true,
  policyUrl: _kMockUrl,
);

final _kMockMarketPlacePolicy = RequestPolicy(
  dataStorageDurationInMinutes: BigInt.from(60 * 24 * 90),
// dataPurpose: 'Registreren',
  dataSharedWithThirdParties: false,
  dataDeletionPossible: true,
  policyUrl: _kMockUrl,
);

final _kMockBarPolicy = RequestPolicy(
  dataStorageDurationInMinutes: BigInt.zero,
// dataPurpose: 'Leeftijd controle',
  dataSharedWithThirdParties: false,
  dataDeletionPossible: false,
  policyUrl: _kMockUrl,
);

final _kMockCarRentalPolicy = RequestPolicy(
  dataStorageDurationInMinutes: BigInt.from(60 * 24 * 90),
// dataPurpose: 'Rijvaardigheid',
  dataSharedWithThirdParties: false,
  dataDeletionPossible: true,
  policyUrl: _kMockUrl,
);

final _kMockFirstAidPolicy = RequestPolicy(
  dataStorageDurationInMinutes: BigInt.from(60 * 24 * 365),
// dataPurpose: 'Zorgverlening',
  dataSharedWithThirdParties: true,
  dataDeletionPossible: true,
  policyUrl: _kMockUrl,
);

final _kMockMunicipalityGenericPolicy = RequestPolicy(
  dataStorageDurationInMinutes: BigInt.from(60 * 24 * 90),
// dataPurpose: 'Gegevens dienen uitsluitend als bewijs',
  dataSharedWithThirdParties: false,
  dataDeletionPossible: true,
  policyUrl: _kMockUrl,
);

final _kMockBankPolicy = RequestPolicy(
  dataStorageDurationInMinutes: BigInt.from(60 * 24 * 90),
// dataPurpose: 'Gegevens dienen uitsluitend als bewijs',
  dataSharedWithThirdParties: false,
  dataDeletionPossible: true,
  policyUrl: _kMockUrl,
);

final _kMockHousingCorpPolicy = RequestPolicy(
  dataStorageDurationInMinutes: BigInt.from(60 * 24 * 90),
// dataPurpose: 'Gegevens dienen uitsluitend als bewijs',
  dataSharedWithThirdParties: false,
  dataDeletionPossible: true,
  policyUrl: _kMockUrl,
);

final _kMonkeyBikePolicy = RequestPolicy(
  dataStorageDurationInMinutes: BigInt.from(60 * 24 * 90),
// dataPurpose: 'Gegevens worden ook gebruikt voor andere doelen',
// dataPurposeDescription: 'De gegevens kunnen worden gebruikt voor marketing en personalisatie.',
  dataSharedWithThirdParties: true,
  dataDeletionPossible: true,
  policyUrl: _kMockUrl,
);

final _kMunicipalityAmsterdamPolicy = RequestPolicy(
  dataStorageDurationInMinutes: BigInt.from(60 * 24 * 365),
  dataSharedWithThirdParties: false,
  dataDeletionPossible: false,
  policyUrl: 'https://www.amsterdam.nl/privacy',
);

// endregion
