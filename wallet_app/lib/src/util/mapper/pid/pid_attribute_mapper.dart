import 'package:collection/collection.dart';
import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/attribute/ui_attribute.dart';
import '../../extension/build_context_extension.dart';
import '../../formatter/attribute_value_formatter.dart';
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
    final birthName = getBirthName(context, input);
    //NOTE: We use the untranslated constructor here, since this function is called with a fresh context
    //NOTE: on every locale change, and thus the correct localization is provided by default.
    return [
      UiAttribute.untranslated(
        value: StringValue(getFullName(context, input)),
        icon: Icons.portrait_outlined,
        label: l10n.walletPersonalizeCheckDataOfferingPageNameLabel,
      ),
      birthName == null
          ? null
          : UiAttribute.untranslated(
              value: StringValue(birthName),
              icon: Icons.crib_outlined,
              label: l10n.walletPersonalizeCheckDataOfferingPageBirthNameLabel,
            ),
      UiAttribute.untranslated(
        value: StringValue(getBirthDetails(context, input)),
        icon: Icons.cake_outlined,
        label: l10n.walletPersonalizeCheckDataOfferingPageBirthInfoLabel,
      ),
      UiAttribute.untranslated(
        value: StringValue(getGender(context, input)),
        icon: Icons.sentiment_satisfied, //FIXME: This icon should probably become dynamic in the future
        label: l10n.walletPersonalizeCheckDataOfferingPageGenderLabel,
      ),
      UiAttribute.untranslated(
        label: l10n.walletPersonalizeCheckDataOfferingPageNationalityLabel,
        value: StringValue(getNationality(context, input)),
        icon: Icons.language_outlined,
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

  String getBirthDetails(BuildContext context, List<T> attributes) {
    return context.l10n.walletPersonalizeCheckDataOfferingPageBirthInfoValue(
      getBirthCountry(context, attributes),
      getBirthDate(context, attributes),
      getBirthPlace(context, attributes),
    );
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

  String? getBirthName(BuildContext context, List<T> attributes) => findByKey(context, attributes, birthNameKey);

  String getBirthDate(BuildContext context, List<T> attributes) => findByKey(context, attributes, birthDateKey)!;

  String getBirthPlace(BuildContext context, List<T> attributes) => findByKey(context, attributes, birthPlaceKey)!;

  String getBirthCountry(BuildContext context, List<T> attributes) => findByKey(context, attributes, birthCountryKey)!;

  String getGender(BuildContext context, List<T> attributes) => findByKey(context, attributes, genderKey)!;

  String getNationality(BuildContext context, List<T> attributes) => findByKey(context, attributes, nationalityKey)!;

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
