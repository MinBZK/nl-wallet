import 'package:wallet_core/core.dart';

final _kMockBirthDate = CardValue.date(value: '1997-03-11');
final _kMockBirthPlace = CardValue.string(value: 'Delft');
final _kMockFirstNames = CardValue.string(value: 'Willeke Liselotte');
final _kMockLastName = CardValue.string(value: 'De Bruijn');
final _kMockGender = CardValue.string(value: 'Vrouw');

final kMockPidDataAttributes = [
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Voornamen')],
    value: _kMockFirstNames,
    key: 'mock.firstNames',
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Achternaam')],
    value: _kMockLastName,
    key: 'mock.lastName',
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Naam bij geboorte')],
    value: CardValue.string(value: 'Molenaar'),
    key: 'mock.birthName',
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Geslacht')],
    value: _kMockGender,
    key: 'mock.gender',
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Geboortedatum')],
    value: _kMockBirthDate,
    key: 'mock.birthDate',
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Ouder dan 18')],
    value: CardValue.boolean(value: true),
    key: 'mock.olderThan18',
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Geboorteplaats')],
    value: _kMockBirthPlace,
    key: 'mock.birthPlace',
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Geboorteland')],
    value: CardValue.string(value: 'Nederland'),
    key: 'mock.birthCountry',
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Burgerservicenummer (BSN)')],
    value: CardValue.string(value: '******999'),
    key: 'mock.citizenshipNumber',
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Nationaliteit')],
    value: CardValue.string(value: 'Nederlands'),
    key: 'mock.nationality',
  ),
];

final kMockAddressDataAttributes = [
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Straatnaam')],
    value: CardValue.string(value: 'Turfmarkt'),
    key: 'mock.streetName',
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Huisnummer')],
    value: CardValue.string(value: '147'),
    key: 'mock.houseNumber',
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Postcode')],
    value: CardValue.string(value: '2511 DP'),
    key: 'mock.postalCode',
  ),
  CardAttribute(
    labels: [LocalizedString(language: 'nl', value: 'Woonplaats')],
    value: CardValue.string(value: 'Den Haag'),
    key: 'mock.city',
  ),
];
