import '../../../domain/model/attribute/core_attribute.dart';
import 'pid_attributes_mapper.dart';

/// Provides the required [Attribute]s needed by the [PidAttributeMapper] to generate a decorated list based on the
/// previewAttributes that are fetched during the pid issuance flow.
class PidCoreAttributeMapper extends PidAttributeMapper<CoreAttribute> {
  @override
  String getBirthCountry(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'pid.countryOfBirth').value;

  @override
  String getBirthDate(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'pid.birthDate').value;

  @override
  String getBirthName(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'pid.birthName').value;

  @override
  String getBirthPlace(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'pid.birthplace').value;

  @override
  String getCitizenId(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'pid.bsn').value;

  @override
  String getFirstNames(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'pid.firstNames').value;

  @override
  String getHouseNumber(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'pid.houseNumber').value;

  @override
  String getLastName(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'pid.lastName').value;

  @override
  String getNationality(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'pid.nationality').value;

  @override
  String getPostalCode(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'pid.postalCode').value;

  @override
  String getResidence(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'pid.residence').value;

  @override
  String getSex(List<CoreAttribute> attributes) => attributes.firstWhere((element) => element.key == 'pid.sex').value;

  @override
  String getStreetName(List<CoreAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'pid.streetName').value;
}
