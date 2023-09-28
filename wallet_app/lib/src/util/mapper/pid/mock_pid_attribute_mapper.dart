import '../../../domain/model/attribute/data_attribute.dart';
import 'pid_attribute_mapper.dart';

/// Provides the attribute keys for the mock pid
class MockPidAttributeMapper extends PidAttributeMapper<DataAttribute> {
  @override
  String get birthCountryKey => 'mock.birthCountry';

  @override
  String get birthDateKey => 'mock.birthDate';

  @override
  String get birthNameKey => 'mock.birthName';

  @override
  String get birthPlaceKey => 'mock.birthPlace';

  @override
  String get bsnKey => 'mock.citizenshipNumber';

  @override
  String get residenceCityKey => 'mock.city';

  @override
  String get firstNamesKey => 'mock.firstNames';

  @override
  String get genderKey => 'mock.gender';

  @override
  String get residenceHouseNumberKey => 'mock.houseNumber';

  @override
  String get lastNameKey => 'mock.lastName';

  @override
  String get nationalityKey => 'mock.nationality';

  @override
  String get residencePostalCodeKey => 'mock.postalCode';

  @override
  String get residenceStreetNameKey => 'mock.streetName';
}
