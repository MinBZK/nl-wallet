import '../../../domain/model/attribute/core_attribute.dart';
import 'pid_attributes_mapper.dart';

/// Provides the required [Attribute]s needed by the [PidAttributeMapper] to generate a decorated list based on the
/// previewAttributes that are fetched during the pid issuance flow.
class PidCoreAttributeMapper extends PidAttributeMapper<CoreAttribute> {
  @override
  String getBirthCountry(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'birth_country').value;

  @override
  String getBirthDate(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'birth_date').value;

  @override
  String getBirthName(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'family_name_birth').value;

  @override
  String getBirthPlace(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'birth_city').value;

  @override
  String getCitizenId(List<CoreAttribute> attributes) => attributes.firstWhere((element) => element.key == 'bsn').value;

  @override
  String getFirstNames(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'given_name').value;

  @override
  String getHouseNumber(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'resident_house_number').value;

  @override
  String getLastName(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'family_name').value;

  @override
  String getNationality(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'nationality').value;

  @override
  String getPostalCode(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'resident_postal_code').value;

  @override
  String getResidence(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'resident_city').value;

  @override
  String getSex(List<CoreAttribute> attributes) => attributes.firstWhere((element) => element.key == 'gender').value;

  @override
  String getStreetName(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'resident_street').value;
}
