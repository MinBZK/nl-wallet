import 'package:wallet_core/core.dart';

final _kMockBirthDate = AttestationValue_String(value: '1997-03-11');
final _kMockBirthPlace = AttestationValue_String(value: 'Delft');
final _kMockFirstNames = AttestationValue_String(value: 'Willeke Liselotte');
final _kMockLastName = AttestationValue_String(value: 'De Bruijn');

final kMockPidAttestationAttributes = [
  AttestationAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Voornamen'),
      LocalizedString(language: 'en', value: 'First names'),
    ],
    value: _kMockFirstNames,
    key: 'mock.firstNames',
  ),
  AttestationAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Achternaam'),
      LocalizedString(language: 'en', value: 'Surname'),
    ],
    value: _kMockLastName,
    key: 'mock.lastName',
  ),
  AttestationAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Naam bij geboorte'),
      LocalizedString(language: 'en', value: 'Birth name'),
    ],
    value: AttestationValue_String(value: 'Molenaar'),
    key: 'mock.birthName',
  ),
  AttestationAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Geboortedatum'),
      LocalizedString(language: 'en', value: 'Birth date'),
    ],
    value: _kMockBirthDate,
    key: 'mock.birthDate',
  ),
  AttestationAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Ouder dan 18'),
      LocalizedString(language: 'en', value: 'Older than 18'),
    ],
    value: AttestationValue_Boolean(value: true),
    key: 'mock.olderThan18',
  ),
  AttestationAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Geboorteplaats'),
      LocalizedString(language: 'en', value: 'Birthplace'),
    ],
    value: _kMockBirthPlace,
    key: 'mock.birthPlace',
  ),
  AttestationAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Geboorteland'),
      LocalizedString(language: 'en', value: 'Country of birth'),
    ],
    value: AttestationValue_String(value: 'Nederland'),
    key: 'mock.birthCountry',
  ),
  AttestationAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Getrouwd of geregistreerd partnerschap'),
      LocalizedString(language: 'en', value: 'Married or registered partnership'),
    ],
    value: AttestationValue_Boolean(value: true),
    key: 'mock.hasSpouseOrPartner',
  ),
  AttestationAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Burger­service­nummer (BSN)'),
      LocalizedString(language: 'en', value: 'BSN'),
    ],
    value: AttestationValue_String(value: '111222333'),
    key: 'mock.citizenshipNumber',
  ),
];

final kMockAddressAttestationAttributes = [
  AttestationAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Land'),
      LocalizedString(language: 'en', value: 'Country'),
    ],
    value: AttestationValue_String(value: 'Nederland'),
    key: 'mock.country',
  ),
  AttestationAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Straatnaam'),
      LocalizedString(language: 'en', value: 'Street'),
    ],
    value: AttestationValue_String(value: 'Turfmarkt'),
    key: 'mock.streetName',
  ),
  AttestationAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Huisnummer'),
      LocalizedString(language: 'en', value: 'House number'),
    ],
    value: AttestationValue_String(value: '147'),
    key: 'mock.houseNumber',
  ),
  AttestationAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Postcode'),
      LocalizedString(language: 'en', value: 'Postal code'),
    ],
    value: AttestationValue_String(value: '2511 DP'),
    key: 'mock.postalCode',
  ),
  AttestationAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Woonplaats'),
      LocalizedString(language: 'en', value: 'City, town or village'),
    ],
    value: AttestationValue_String(value: 'Den Haag'),
    key: 'mock.city',
  ),
];
