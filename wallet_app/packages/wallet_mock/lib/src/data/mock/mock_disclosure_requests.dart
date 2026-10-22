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
const _kMockCityKey = 'mock_city';
const _kMockStreetNameKey = 'mock_streetName';
const _kMockPostalCodeKey = 'mock_postalCode';
const _kMockHouseNumberKey = 'mock_houseNumber';

final List<DisclosureRequest> kDisclosureRequests = [
  DisclosureRequest(
    id: _kJobApplicationId,
    relyingParty: kOrganizations[kEmployerId]!,
    requestedAttributes: _kJobApplicationRequestedAttributes,
    purpose: 'Sollicitatie',
  ),
  DisclosureRequest(
    id: _kMarketplaceLoginId,
    relyingParty: kOrganizations[kMarketplaceId]!,
    requestedAttributes: _kMarketplaceLoginRequestedAttributes,
    purpose: 'Account aanmaken',
  ),
  DisclosureRequest(
    id: _kBarId,
    relyingParty: kOrganizations[kBarId]!,
    requestedAttributes: _kBarRequestedAttributes,
    purpose: 'Leeftijd controle',
  ),
  DisclosureRequest(
    id: _kCarRental,
    relyingParty: kOrganizations[kCarRentalId]!,
    requestedAttributes: _kCarRentalRequestedAttributes,
    purpose: 'Gegevens & Rijbewijs controle',
  ),
  DisclosureRequest(
    id: _kFirstAid,
    relyingParty: kOrganizations[kFirstAidId]!,
    requestedAttributes: _kFirstAidRequestedAttributes,
    purpose: 'Gegevens & Verzekering controle',
  ),
  DisclosureRequest(
    id: _kParkingPermit,
    relyingParty: kOrganizations[kMunicipalityTheHagueId]!,
    requestedAttributes: _kParkingPermitRequestedAttributes,
    purpose: 'Parkeervergunning',
  ),
  DisclosureRequest(
    id: _kOpenBankAccount,
    relyingParty: kOrganizations[kBankId]!,
    requestedAttributes: _kOpenBankAccountRequestedAttributes,
    purpose: 'Rekening openen',
  ),
  DisclosureRequest(
    id: _kProvideContractDetails,
    relyingParty: kOrganizations[kHousingCorpId]!,
    requestedAttributes: _kProvideContractDetailsRequestedAttributes,
    purpose: 'Identificeren',
  ),
  DisclosureRequest(
    id: _kCreateMonkeyBikeAccount,
    relyingParty: kOrganizations[kMonkeyBikeId]!,
    requestedAttributes: _kCreateMbAccountRequestedAttributes,
    purpose: 'Account aanmaken',
  ),
  DisclosureRequest(
    id: _kPharmacy,
    relyingParty: kOrganizations[kPharmacyId]!,
    requestedAttributes: _kPharmacyRequestedAttributes,
    purpose: 'Herhaalrecept',
  ),
  DisclosureRequest(
    id: _kAmsterdamLoginId,
    relyingParty: kOrganizations[kMunicipalityAmsterdamId]!,
    requestedAttributes: _kCitizenshipNumberRequest,
    purpose: 'Inloggen',
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
    label: 'Stad',
    key: _kMockCityKey,
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
