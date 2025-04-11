import 'package:wallet_core/core.dart';

const kMockCitizenShipNumberKey = 'mock_citizenshipNumber';
const kMockFirstNamesKey = 'mock_firstNames';
const kMockLastNameKey = 'mock_lastName';
const kMockBirthDateKey = 'mock_birthDate';
const kMockOtherKey = 'mock_other';
const kMockIssuanceDateKey = 'mock_issuanceDate';

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
    key: 'mock_firstNames',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Achternaam'),
      ClaimDisplayMetadata(lang: 'en', label: 'Surname'),
    ],
    value: _kMockLastName,
    key: 'mock_lastName',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Naam bij geboorte'),
      ClaimDisplayMetadata(lang: 'en', label: 'Birth name'),
    ],
    value: AttributeValue_String(value: 'Molenaar'),
    key: 'mock_birthName',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Geboortedatum'),
      ClaimDisplayMetadata(lang: 'en', label: 'Birth date'),
    ],
    value: _kMockBirthDate,
    key: 'mock_birthDate',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Ouder dan 18'),
      ClaimDisplayMetadata(lang: 'en', label: 'Older than 18'),
    ],
    value: AttributeValue_Boolean(value: true),
    key: 'mock_olderThan18',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Geboorteplaats'),
      ClaimDisplayMetadata(lang: 'en', label: 'Birthplace'),
    ],
    value: _kMockBirthPlace,
    key: 'mock_birthPlace',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Geboorteland'),
      ClaimDisplayMetadata(lang: 'en', label: 'Country of birth'),
    ],
    value: AttributeValue_String(value: 'Nederland'),
    key: 'mock_birthCountry',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Getrouwd of geregistreerd partnerschap'),
      ClaimDisplayMetadata(lang: 'en', label: 'Married or registered partnership'),
    ],
    value: AttributeValue_Boolean(value: true),
    key: 'mock_hasSpouseOrPartner',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Burger­service­nummer (BSN)'),
      ClaimDisplayMetadata(lang: 'en', label: 'BSN'),
    ],
    value: AttributeValue_String(value: '111222333'),
    key: kMockCitizenShipNumberKey,
  ),
];

final kMockAddressAttestationAttributes = [
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Land'),
      ClaimDisplayMetadata(lang: 'en', label: 'Country'),
    ],
    value: AttributeValue_String(value: 'Nederland'),
    key: 'mock_country',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Straatnaam'),
      ClaimDisplayMetadata(lang: 'en', label: 'Street'),
    ],
    value: AttributeValue_String(value: 'Turfmarkt'),
    key: 'mock_streetName',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Huisnummer'),
      ClaimDisplayMetadata(lang: 'en', label: 'House number'),
    ],
    value: AttributeValue_String(value: '147'),
    key: 'mock_houseNumber',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Postcode'),
      ClaimDisplayMetadata(lang: 'en', label: 'Postal code'),
    ],
    value: AttributeValue_String(value: '2511 DP'),
    key: 'mock_postalCode',
  ),
  AttestationAttribute(
    labels: [
      ClaimDisplayMetadata(lang: 'nl', label: 'Woonplaats'),
      ClaimDisplayMetadata(lang: 'en', label: 'City, town or village'),
    ],
    value: AttributeValue_String(value: 'Den Haag'),
    key: 'mock_city',
  ),
];

final kMockDiplomaAttestationAttributes = [
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Onderwijsinstelling')],
    value: const AttributeValue.string(value: 'Universiteit X'),
    key: 'mock_university',
    // sourceCardId: _kDiplomaId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Opleiding')],
    value: const AttributeValue.string(value: 'WO Bachelor Bedrijfskunde'),
    key: 'mock_education',
    // sourceCardId: _kDiplomaId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Niveau')],
    value: const AttributeValue.string(value: 'WO'),
    key: 'mock_educationLevel',
    // sourceCardId: _kDiplomaId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Type')],
    value: const AttributeValue.string(value: 'Getuigschrift'),
    // sourceCardId: _kDiplomaId,
    key: kMockOtherKey,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Uitgifte datum')],
    value: AttributeValue.string(value: '01-01-2013'),
    key: kMockIssuanceDateKey,
    // sourceCardId: _kDiplomaId,
  ),
];

final kMockMasterDiplomaDataAttributes = [
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Onderwijsinstelling')],
    value: const AttributeValue.string(value: 'Universiteit X'),
    key: 'mock_university',
    // sourceCardId: _kMasterDiplomaId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Opleiding')],
    value: const AttributeValue.string(value: 'WO Master Bedrijfskunde'),
    key: 'mock_education',
    // sourceCardId: _kMasterDiplomaId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Niveau')],
    value: const AttributeValue.string(value: 'WO'),
    key: 'mock_educationLevel',
    // sourceCardId: _kMasterDiplomaId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Type')],
    value: const AttributeValue.string(value: 'Getuigschrift'),
    // sourceCardId: _kMasterDiplomaId,
    key: kMockOtherKey,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Uitgifte datum')],
    value: AttributeValue.string(value: '01-01-2015'),
    key: kMockIssuanceDateKey,
    // sourceCardId: _kMasterDiplomaId,
  ),
];

final kMockDrivingLicenseDataAttributes = _buildDrivingLicenseDataAttributes(category: 'AM, B, BE');
final kMockDrivingLicenseRenewedDataAttributes = _buildDrivingLicenseDataAttributes(category: 'AM, B, C1, BE');

List<AttestationAttribute> _buildDrivingLicenseDataAttributes({required String category}) {
  return [
    AttestationAttribute(
      labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Voornamen')],
      value: _kMockFirstNames,
      key: kMockFirstNamesKey,
      // sourceCardId: _kDrivingLicenseId,
    ),
    AttestationAttribute(
      labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Naam')],
      value: _kMockLastName,
      key: kMockLastNameKey,
      // sourceCardId: _kDrivingLicenseId,
    ),
    AttestationAttribute(
      labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Geboortedatum')],
      value: _kMockBirthDate,
      key: kMockBirthDateKey,
      // sourceCardId: _kDrivingLicenseId,
    ),
    AttestationAttribute(
      labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Geboorteplaats')],
      value: _kMockBirthPlace,
      key: 'mock_birthPlace',
      // sourceCardId: _kDrivingLicenseId,
    ),
    AttestationAttribute(
      labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Afgiftedatum')],
      value: AttributeValue.string(value: '04-23-2018'),
      key: kMockIssuanceDateKey,
      // sourceCardId: _kDrivingLicenseId,
    ),
    AttestationAttribute(
      labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Datum geldig tot')],
      value: AttributeValue.string(value: '23-04-2028'),
      key: 'mock_expiryDate',
      // sourceCardId: _kDrivingLicenseId,
    ),
    AttestationAttribute(
      labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Rijbewijsnummer')],
      value: const AttributeValue.string(value: '99999999999'),
      // sourceCardId: _kDrivingLicenseId,
      key: kMockOtherKey,
    ),
    AttestationAttribute(
      labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Rijbewijscategorieën')],
      value: AttributeValue.string(value: category),
      key: 'mock_drivingLicenseCategories',
      // sourceCardId: _kDrivingLicenseId,
    ),
  ];
}

final kMockHealthInsuranceDataAttributes = [
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Naam')],
    value: AttributeValue.string(value: 'Willeke De Bruijn'),
    key: 'mock_fullName',
    // sourceCardId: _kHealthInsuranceId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Geslacht')],
    value: AttributeValue.string(value: 'Vrouw'),
    key: 'mock_gender',
    // sourceCardId: _kHealthInsuranceId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Geboortedatum')],
    value: _kMockBirthDate,
    key: kMockBirthDateKey,
    // sourceCardId: _kHealthInsuranceId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Klantnummer')],
    value: const AttributeValue.string(value: '12345678'),
    key: 'mock_healthIssuerClientId',
    // sourceCardId: _kHealthInsuranceId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Kaartnummer')],
    value: const AttributeValue.string(value: '9999999999'),
    key: 'mock_documentNr',
    // sourceCardId: _kHealthInsuranceId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'UZOVI')],
    value: const AttributeValue.string(value: 'XXXX - 9999'),
    key: 'mock_healthIssuerId',
    // sourceCardId: _kHealthInsuranceId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Verloopdatum')],
    value: AttributeValue.string(value: '0-01-2024'),
    key: 'mock_healthInsuranceExpiryDate',
    // sourceCardId: _kHealthInsuranceId,
  ),
];

final kMockVOGDataAttributes = [
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Type')],
    value: const AttributeValue.string(value: '1'),
    key: 'mock_certificateOfConduct',
    // sourceCardId: _kVOGId,
  ),
  AttestationAttribute(
    labels: [ClaimDisplayMetadata(lang: 'nl', label: 'Datum geldig tot')],
    value: AttributeValue.string(value: '05-02-2023'),
    key: 'mock_expiryDate',
    // sourceCardId: _kVOGId,
  ),
];
