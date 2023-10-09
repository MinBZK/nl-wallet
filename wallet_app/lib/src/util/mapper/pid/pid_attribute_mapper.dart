import 'package:collection/collection.dart';
import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/attribute/ui_attribute.dart';
import '../../extension/build_context_extension.dart';
import '../context_mapper.dart';

/// Mapper that takes a list of attributes and turns them into a list of decorated [UiAttribute]s.
abstract class PidAttributeMapper<T extends Attribute> extends ContextMapper<List<T>, List<UiAttribute>> {
  String get firstNamesKey;

  String get lastNameKey;

  String get birthNameKey;

  String get birthCountryKey;

  String get birthDateKey;

  String get birthPlaceKey;

  String get genderKey;

  String get nationalityKey;

  String get bsnKey;

  String get residenceStreetNameKey;

  String get residenceHouseNumberKey;

  String get residencePostalCodeKey;

  String get residenceCityKey;

  @override
  List<UiAttribute> map(BuildContext context, List<T> input) {
    final l10n = context.l10n;
    final birthName = getBirthName(input);
    return [
      UiAttribute(
        value: getFullName(input),
        icon: Icons.portrait_outlined,
        label: l10n.walletPersonalizeCheckDataOfferingPageNameLabel,
      ),
      birthName == null
          ? null
          : UiAttribute(
              value: birthName,
              icon: Icons.crib_outlined,
              label: l10n.walletPersonalizeCheckDataOfferingPageBirthNameLabel,
            ),
      UiAttribute(
        value: getBirthDetails(context, input),
        icon: Icons.cake_outlined,
        label: l10n.walletPersonalizeCheckDataOfferingPageBirthInfoLabel,
      ),
      UiAttribute(
        value: getGender(input),
        icon: Icons.sentiment_satisfied, //FIXME: This icon should probably become dynamic in the future
        label: l10n.walletPersonalizeCheckDataOfferingPageGenderLabel,
      ),
      UiAttribute(
        label: l10n.walletPersonalizeCheckDataOfferingPageNationalityLabel,
        value: getNationality(input),
        icon: Icons.language_outlined,
      ),
      UiAttribute(
        label: l10n.walletPersonalizeCheckDataOfferingPageCitizenIdLabel,
        value: getBsn(input),
        icon: Icons.badge_outlined,
      ),
      UiAttribute(
        label: l10n.walletPersonalizeCheckDataOfferingPageAddressLabel,
        value: getResidentialAddress(input),
        icon: Icons.cottage_outlined,
      ),
    ].nonNulls.toList();
  }

  String getBirthDetails(BuildContext context, List<T> attributes) {
    return context.l10n.walletPersonalizeCheckDataOfferingPageBirthInfoValue(
      getBirthCountry(attributes),
      getBirthDate(attributes),
      getBirthPlace(attributes),
    );
  }

  String getResidentialAddress(List<T> attributes) {
    final streetName = getStreetName(attributes);
    final houseNumber = getHouseNumber(attributes);
    final postalCode = getPostalCode(attributes);
    final city = getCity(attributes);
    return '$streetName $houseNumber, $postalCode $city';
  }

  String getFullName(List<T> attributes) => '${getFirstNames(attributes)} ${getLastName(attributes)}';

  String getFirstNames(List<T> attributes) => findByKey(attributes, firstNamesKey)!;

  String getLastName(List<T> attributes) => findByKey(attributes, lastNameKey)!;

  String? getBirthName(List<T> attributes) => findByKey(attributes, birthNameKey);

  String getBirthDate(List<T> attributes) => findByKey(attributes, birthDateKey)!;

  String getBirthPlace(List<T> attributes) => findByKey(attributes, birthPlaceKey)!;

  String getBirthCountry(List<T> attributes) => findByKey(attributes, birthCountryKey)!;

  String getGender(List<T> attributes) => findByKey(attributes, genderKey)!;

  String getNationality(List<T> attributes) => findByKey(attributes, nationalityKey)!;

  String getBsn(List<T> attributes) => findByKey(attributes, bsnKey)!;

  String getCity(List<T> attributes) => findByKey(attributes, residenceCityKey)!;

  String getPostalCode(List<T> attributes) => findByKey(attributes, residencePostalCodeKey)!;

  String getStreetName(List<T> attributes) => findByKey(attributes, residenceStreetNameKey)!;

  String getHouseNumber(List<T> attributes) => findByKey(attributes, residenceHouseNumberKey)!;

  String? findByKey(List<Attribute> attributes, String key) {
    final attribute = attributes.firstWhereOrNull((attribute) => attribute.key == key);
    if (attribute == null) return null;
    if (attribute is DataAttribute) return attribute.value;
    throw UnimplementedError('Value could not be extracted from attribute with type: ${attribute.valueType}');
  }
}
