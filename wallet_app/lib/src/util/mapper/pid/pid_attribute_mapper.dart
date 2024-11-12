import 'package:collection/collection.dart';
import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/attribute/ui_attribute.dart';
import '../../../domain/model/attribute/value/gender.dart';
import '../../extension/build_context_extension.dart';
import '../../formatter/attribute_value_formatter.dart';
import '../context_mapper.dart';

/// Mapper that takes a list of attributes and turns them into a list of decorated [UiAttribute]s.
abstract class PidAttributeMapper<T extends Attribute> extends ContextMapper<List<T>, List<UiAttribute>> {
  String get firstNamesKey;

  String get lastNameKey;

  String get birthCountryKey;

  String get birthDateKey;

  String get birthCityKey;

  String get genderKey;

  String get bsnKey;

  String get residenceStreetNameKey;

  String get residenceHouseNumberKey;

  String get residencePostalCodeKey;

  String get residenceCityKey;

  @override
  List<UiAttribute> map(BuildContext context, List<T> input) {
    final l10n = context.l10n;
    //NOTE: We use the untranslated constructor here, since this function is called with a fresh context
    //NOTE: on every locale change, and thus the correct localization is provided by default.
    return [
      UiAttribute.untranslated(
        label: l10n.walletPersonalizeCheckDataOfferingPageNameLabel,
        value: StringValue(getFullName(context, input)),
        icon: Icons.portrait_outlined,
      ),
      UiAttribute.untranslated(
        label: l10n.walletPersonalizeCheckDataOfferingPageBirthInfoLabel,
        value: StringValue(getBirthDetails(context, input)),
        icon: Icons.cake_outlined,
      ),
      // Gender is a temporary optional attribute due to the current 'autorisatiebesluit'.
      // See https://SSSS/browse/PVW-3642 for more information.
      if (getGender(context, input) != null)
        UiAttribute.untranslated(
          label: context.l10n.walletPersonalizeCheckDataOfferingPageGenderLabel,
          value: StringValue(getGender(context, input)!),
          icon: getGenderIcon(input),
        ),
      UiAttribute.untranslated(
        label: l10n.walletPersonalizeCheckDataOfferingPageCitizenIdLabel,
        value: StringValue(getBsn(context, input)),
        icon: Icons.badge_outlined,
      ),
      UiAttribute.untranslated(
        label: l10n.walletPersonalizeCheckDataOfferingPageAddressLabel,
        value: StringValue(getResidentialAddress(context, input)),
        icon: Icons.cottage_outlined,
      ),
    ].nonNulls.toList();
  }

  IconData getGenderIcon(List<T> input) {
    final gender = getGenderValue(input).value;
    return switch (gender) {
      Gender.unknown => Icons.question_mark_outlined,
      Gender.male => Icons.male_outlined,
      Gender.female => Icons.female_outlined,
      Gender.notApplicable => Icons.sentiment_satisfied,
    };
  }

  String getBirthDetails(BuildContext context, List<T> attributes) {
    final birthCountry = getBirthCountry(context, attributes);
    final birthCity = getBirthCity(context, attributes);
    final birthDate = getBirthDate(context, attributes);
    if (birthCountry != null && birthCity != null) {
      return context.l10n.walletPersonalizeCheckDataOfferingPageBirthInfoValue(
        birthCountry,
        birthDate,
        birthCity,
      );
    } else {
      return birthDate;
    }
  }

  String getResidentialAddress(BuildContext context, List<T> attributes) {
    final streetName = getStreetName(context, attributes);
    final houseNumber = getHouseNumber(context, attributes);
    final postalCode = getPostalCode(context, attributes);
    final city = getCity(context, attributes);
    return '$streetName $houseNumber, $postalCode $city';
  }

  String getFullName(BuildContext context, List<T> attributes) =>
      '${getFirstNames(context, attributes)} ${getLastName(context, attributes)}';

  String getFirstNames(BuildContext context, List<T> attributes) => findByKey(context, attributes, firstNamesKey)!;

  String getLastName(BuildContext context, List<T> attributes) => findByKey(context, attributes, lastNameKey)!;

  String getBirthDate(BuildContext context, List<T> attributes) => findByKey(context, attributes, birthDateKey)!;

  String? getBirthCity(BuildContext context, List<T> attributes) => findByKey(context, attributes, birthCityKey);

  String? getBirthCountry(BuildContext context, List<T> attributes) => findByKey(context, attributes, birthCountryKey);

  String? getGender(BuildContext context, List<T> attributes) => findByKey(context, attributes, genderKey);

  GenderValue getGenderValue(List<T> attributes) =>
      attributes.whereType<DataAttribute>().firstWhere((attribute) => attribute.key == genderKey).value as GenderValue;

  String getBsn(BuildContext context, List<T> attributes) => findByKey(context, attributes, bsnKey)!;

  String getCity(BuildContext context, List<T> attributes) => findByKey(context, attributes, residenceCityKey)!;

  String getPostalCode(BuildContext context, List<T> attributes) =>
      findByKey(context, attributes, residencePostalCodeKey)!;

  String getStreetName(BuildContext context, List<T> attributes) =>
      findByKey(context, attributes, residenceStreetNameKey)!;

  String getHouseNumber(BuildContext context, List<T> attributes) =>
      findByKey(context, attributes, residenceHouseNumberKey)!;

  String? findByKey(BuildContext context, List<Attribute> attributes, String key) {
    final attribute = attributes.firstWhereOrNull((attribute) => attribute.key == key);
    if (attribute == null) return null;
    if (attribute is DataAttribute) return attribute.value.prettyPrint(context);
    throw UnimplementedError('Value could not be extracted from attribute: $attribute}');
  }
}
