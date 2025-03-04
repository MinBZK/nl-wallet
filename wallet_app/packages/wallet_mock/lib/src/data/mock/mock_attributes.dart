import 'package:wallet_core/core.dart';

final _kMockBirthDate = AttributeValue_String(value: '1997-03-11');
final _kMockBirthPlace = AttributeValue_String(value: 'Delft');
final _kMockFirstNames = AttributeValue_String(value: 'Willeke Liselotte');
final _kMockLastName = AttributeValue_String(value: 'De Bruijn');

final kMockPidAttestationAttributes = [
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Voornamen'),
      ClaimDisplayMetadata(lang: 'en', label: 'First names'),
    ],
    value: _kMockFirstNames,
    key: 'mock.firstNames',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Achternaam'),
      ClaimDisplayMetadata(lang: 'en', label: 'Surname'),
    ],
    value: _kMockLastName,
    key: 'mock.lastName',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Naam bij geboorte'),
      ClaimDisplayMetadata(lang: 'en', label: 'Birth name'),
    ],
    value: AttributeValue_String(value: 'Molenaar'),
    key: 'mock.birthName',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Geboortedatum'),
      ClaimDisplayMetadata(lang: 'en', label: 'Birth date'),
    ],
    value: _kMockBirthDate,
    key: 'mock.birthDate',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Ouder dan 18'),
      ClaimDisplayMetadata(lang: 'en', label: 'Older than 18'),
    ],
    value: AttributeValue_Boolean(value: true),
    key: 'mock.olderThan18',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Geboorteplaats'),
      ClaimDisplayMetadata(lang: 'en', label: 'Birthplace'),
    ],
    value: _kMockBirthPlace,
    key: 'mock.birthPlace',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Geboorteland'),
      ClaimDisplayMetadata(lang: 'en', label: 'Country of birth'),
    ],
    value: AttributeValue_String(value: 'Nederland'),
    key: 'mock.birthCountry',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Getrouwd of geregistreerd partnerschap'),
      ClaimDisplayMetadata(lang: 'en', label: 'Married or registered partnership'),
    ],
    value: AttributeValue_Boolean(value: true),
    key: 'mock.hasSpouseOrPartner',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Burger­service­nummer (BSN)'),
      ClaimDisplayMetadata(lang: 'en', label: 'BSN'),
    ],
    value: AttributeValue_String(value: '111222333'),
    key: 'mock.citizenshipNumber',
  ),
];

final kMockAddressAttestationAttributes = [
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Land'),
      ClaimDisplayMetadata(lang: 'en', label: 'Country'),
    ],
    value: AttributeValue_String(value: 'Nederland'),
    key: 'mock.country',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Straatnaam'),
      ClaimDisplayMetadata(lang: 'en', label: 'Street'),
    ],
    value: AttributeValue_String(value: 'Turfmarkt'),
    key: 'mock.streetName',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Huisnummer'),
      ClaimDisplayMetadata(lang: 'en', label: 'House number'),
    ],
    value: AttributeValue_String(value: '147'),
    key: 'mock.houseNumber',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Postcode'),
      ClaimDisplayMetadata(lang: 'en', label: 'Postal code'),
    ],
    value: AttributeValue_String(value: '2511 DP'),
    key: 'mock.postalCode',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Woonplaats'),
      ClaimDisplayMetadata(lang: 'en', label: 'City, town or village'),
    ],
    value: AttributeValue_String(value: 'Den Haag'),
    key: 'mock.city',
  ),
];
