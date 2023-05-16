import 'package:flutter/material.dart';

import '../../domain/model/attribute/data_attribute.dart';
import '../../domain/model/attribute/ui_attribute.dart';

/// This class makes some pretty harsh assumptions as to what is available in the
/// provided attributes. Since what the actual model and PID will look like is still
/// unknown these assumptions kept for simplicity sake.
class PidAttributeMapper {
  static List<UiAttribute> map(List<DataAttribute> attributes) {
    return [
      _resolveNameAttribute(attributes),
      _resolveBirthNameAttribute(attributes),
      _resolveBirthDetailsAttribute(attributes),
      _resolveGenderAttribute(attributes),
      _resolveNationalityAttribute(attributes),
      _resolveCitizenIdAttribute(attributes),
      _resolveAddressAttribute(attributes),
    ];
  }

  static UiAttribute _resolveNameAttribute(List<DataAttribute> attributes) {
    final firstName = attributes.firstWhere((element) => element.type == AttributeType.firstNames);
    final lastName = attributes.firstWhere((element) => element.type == AttributeType.lastName);
    return UiAttribute(
      label: 'Naam',
      value: '${firstName.value} ${lastName.value}',
      icon: Icons.portrait_outlined,
    );
  }

  static UiAttribute _resolveBirthNameAttribute(List<DataAttribute> attributes) {
    final attrib = attributes.firstWhere((element) => element.type == AttributeType.birthName);
    return UiAttribute(
      label: attrib.label,
      value: attrib.value,
      icon: Icons.crib_outlined,
    );
  }

  static UiAttribute _resolveBirthDetailsAttribute(List<DataAttribute> attributes) {
    final birthDate = attributes.firstWhere((element) => element.type == AttributeType.birthDate);
    final birthPlace = attributes.firstWhere((element) => element.type == AttributeType.birthPlace);
    final birthCountry = attributes.firstWhere((element) => element.type == AttributeType.birthCountry);
    return UiAttribute(
      label: 'Geboren',
      value: '${birthDate.value} in ${birthPlace.value}, ${birthCountry.value}',
      icon: Icons.cake_outlined,
    );
  }

  static UiAttribute _resolveGenderAttribute(List<DataAttribute> attributes) {
    final attrib = attributes.firstWhere((element) => element.type == AttributeType.gender);
    return UiAttribute(
      label: attrib.label,
      value: attrib.value,
      icon: Icons.female_outlined,
    );
  }

  static UiAttribute _resolveNationalityAttribute(List<DataAttribute> attributes) {
    final attrib = attributes.firstWhere((element) => element.type == AttributeType.nationality);
    return UiAttribute(
      label: attrib.label,
      value: attrib.value,
      icon: Icons.language_outlined,
    );
  }

  static UiAttribute _resolveCitizenIdAttribute(List<DataAttribute> attributes) {
    final attrib = attributes.firstWhere((element) => element.type == AttributeType.citizenshipNumber);
    return UiAttribute(
      label: attrib.label,
      value: attrib.value,
      icon: Icons.badge_outlined,
    );
  }

  static UiAttribute _resolveAddressAttribute(List<DataAttribute> attributes) {
    final city = attributes.firstWhere((element) => element.type == AttributeType.city);
    final postalCode = attributes.firstWhere((element) => element.type == AttributeType.postalCode);
    final streetName = attributes.firstWhere((element) => element.type == AttributeType.streetName);
    final houseNumber = attributes.firstWhere((element) => element.type == AttributeType.houseNumber);
    return UiAttribute(
      label: 'Adres',
      value: '${streetName.value} ${houseNumber.value}, ${postalCode.value} ${city.value}',
      icon: Icons.cottage_outlined,
    );
  }
}
