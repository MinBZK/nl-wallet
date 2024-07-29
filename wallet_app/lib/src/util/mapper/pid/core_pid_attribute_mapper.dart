import '../../../domain/model/attribute/data_attribute.dart';
import 'pid_attribute_mapper.dart';

/// Provides the attribute keys for the core pid
class CorePidAttributeMapper extends PidAttributeMapper<DataAttribute> {
  @override
  String get birthCountryKey => 'birth_country';

  @override
  String get birthDateKey => 'birth_date';

  @override
  String get birthCityKey => 'birth_city';

  @override
  String get bsnKey => 'bsn';

  @override
  String get residenceCityKey => 'resident_city';

  @override
  String get firstNamesKey => 'given_name';

  @override
  String get genderKey => 'gender';

  @override
  String get residenceHouseNumberKey => 'resident_house_number';

  @override
  String get lastNameKey => 'family_name';

  @override
  String get residencePostalCodeKey => 'resident_postal_code';

  @override
  String get residenceStreetNameKey => 'resident_street';
}
