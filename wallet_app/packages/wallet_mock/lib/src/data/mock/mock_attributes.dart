import 'package:wallet_core/core.dart';

final _kMockBirthDate = CardValue.date(value: '1997-03-11');
final _kMockBirthPlace = CardValue.string(value: 'Delft');
final _kMockFirstNames = CardValue.string(value: 'Willeke Liselotte');
final _kMockLastName = CardValue.string(value: 'De Bruijn');
final _kMockGender = CardValue.gender(value: GenderCardValue.Female);

final kMockPidDataAttributes = [
  CardAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Voornamen'),
      LocalizedString(language: 'en', value: 'First names'),
    ],
    value: _kMockFirstNames,
    key: 'mock.firstNames',
  ),
  CardAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Achternaam'),
      LocalizedString(language: 'en', value: 'Surname'),
    ],
    value: _kMockLastName,
    key: 'mock.lastName',
  ),
  CardAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Naam bij geboorte'),
      LocalizedString(language: 'en', value: 'Birth name'),
    ],
    value: CardValue.string(value: 'Molenaar'),
    key: 'mock.birthName',
  ),
  CardAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Geslacht'),
      LocalizedString(language: 'en', value: 'Gender'),
    ],
    value: _kMockGender,
    key: 'mock.gender',
  ),
  CardAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Geboortedatum'),
      LocalizedString(language: 'en', value: 'Birth date'),
    ],
    value: _kMockBirthDate,
    key: 'mock.birthDate',
  ),
  CardAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Ouder dan 18'),
      LocalizedString(language: 'en', value: 'Older than 18'),
    ],
    value: CardValue.boolean(value: true),
    key: 'mock.olderThan18',
  ),
  CardAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Geboorteplaats'),
      LocalizedString(language: 'en', value: 'Birthplace'),
    ],
    value: _kMockBirthPlace,
    key: 'mock.birthPlace',
  ),
  CardAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Geboorteland'),
      LocalizedString(language: 'en', value: 'Country of birth'),
    ],
    value: CardValue.string(value: 'Nederland'),
    key: 'mock.birthCountry',
  ),
  CardAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Getrouwd of geregistreerd partnerschap'),
      LocalizedString(language: 'en', value: 'Married or registered partnership'),
    ],
    value: CardValue.boolean(value: true),
    key: 'mock.hasSpouseOrPartner',
  ),
  CardAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Burger­service­nummer (BSN)'),
      LocalizedString(language: 'en', value: 'BSN'),
    ],
    value: CardValue.string(value: '111222333'),
    key: 'mock.citizenshipNumber',
  ),
];

final kMockAddressDataAttributes = [
  CardAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Land'),
      LocalizedString(language: 'en', value: 'Country'),
    ],
    value: CardValue.string(value: 'Nederland'),
    key: 'mock.country',
  ),
  CardAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Straatnaam'),
      LocalizedString(language: 'en', value: 'Street'),
    ],
    value: CardValue.string(value: 'Turfmarkt'),
    key: 'mock.streetName',
  ),
  CardAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Huisnummer'),
      LocalizedString(language: 'en', value: 'House number'),
    ],
    value: CardValue.string(value: '147'),
    key: 'mock.houseNumber',
  ),
  CardAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Postcode'),
      LocalizedString(language: 'en', value: 'Postal code'),
    ],
    value: CardValue.string(value: '2511 DP'),
    key: 'mock.postalCode',
  ),
  CardAttribute(
    labels: [
      LocalizedString(language: 'nl', value: 'Woonplaats'),
      LocalizedString(language: 'en', value: 'City, town or village'),
    ],
    value: CardValue.string(value: 'Den Haag'),
    key: 'mock.city',
  ),
];
