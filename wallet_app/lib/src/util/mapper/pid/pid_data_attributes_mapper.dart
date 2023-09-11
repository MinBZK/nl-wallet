import '../../../domain/model/attribute/data_attribute.dart';
import 'pid_attributes_mapper.dart';

/// Provides the required [Attribute]s needed by the [PidAttributeMapper] to generate a decorated list based on the
/// mocked previewAttributes that are retrieved during the pid issuance flow.
class PidDataAttributeMapper extends PidAttributeMapper<DataAttribute> {
  @override
  String getBirthCountry(List<DataAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'mock.birthCountry').value;

  @override
  String getBirthDate(List<DataAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'mock.birthDate').value;

  @override
  String getBirthName(List<DataAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'mock.birthName').value;

  @override
  String getBirthPlace(List<DataAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'mock.birthPlace').value;

  @override
  String getCitizenId(List<DataAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'mock.citizenshipNumber').value;

  @override
  String getFirstNames(List<DataAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'mock.firstNames').value;

  @override
  String getHouseNumber(List<DataAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'mock.houseNumber').value;

  @override
  String getLastName(List<DataAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'mock.lastName').value;

  @override
  String getNationality(List<DataAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'mock.nationality').value;

  @override
  String getPostalCode(List<DataAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'mock.postalCode').value;

  @override
  String getResidence(List<DataAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'mock.city').value;

  @override
  String getSex(List<DataAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'mock.gender').value;

  @override
  String getStreetName(List<DataAttribute> attributes) =>
      attributes.firstWhere((element) => element.key == 'mock.streetName').value;
}
