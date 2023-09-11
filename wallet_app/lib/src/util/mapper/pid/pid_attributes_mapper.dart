import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/attribute/ui_attribute.dart';
import '../../extension/build_context_extension.dart';

/// Mapper that takes a list of attributes and turns them into a list of decorated [UiAttribute]s.
abstract class PidAttributeMapper<T extends Attribute> {
  List<UiAttribute> map(BuildContext context, List<T> attributes) {
    final l10n = context.l10n;
    return [
      UiAttribute(
        value: getFullName(attributes),
        icon: Icons.portrait_outlined,
        label: l10n.walletPersonalizeCheckDataOfferingPageNameLabel,
      ),
      UiAttribute(
        value: getBirthName(attributes),
        icon: Icons.crib_outlined,
        label: l10n.walletPersonalizeCheckDataOfferingPageNameLabel,
      ),
      UiAttribute(
        value: getBirthDetails(context, attributes),
        icon: Icons.cake_outlined,
        label: l10n.walletPersonalizeCheckDataOfferingPageBirthInfoLabel,
      ),
      UiAttribute(
        value: getSex(attributes),
        icon: Icons.female_outlined, //FIXME: This icon should probably become dynamic in the future
        label: l10n.walletPersonalizeCheckDataOfferingPageBirthInfoLabel,
      ),
      UiAttribute(
        label: l10n.walletPersonalizeCheckDataOfferingPageNationalityLabel,
        value: getNationality(attributes),
        icon: Icons.language_outlined,
      ),
      UiAttribute(
        label: l10n.walletPersonalizeCheckDataOfferingPageCitizenIdLabel,
        value: getCitizenId(attributes),
        icon: Icons.badge_outlined,
      ),
      UiAttribute(
        label: l10n.walletPersonalizeCheckDataOfferingPageAddressLabel,
        value: getFullAddress(attributes),
        icon: Icons.cottage_outlined,
      ),
    ];
  }

  String getFullName(List<T> attributes) => '${getFirstNames(attributes)} ${getLastName(attributes)}';

  String getFirstNames(List<T> attributes);

  String getLastName(List<T> attributes);

  String getBirthName(List<T> attributes);

  String getBirthDetails(BuildContext context, List<T> attributes) {
    return context.l10n.walletPersonalizeCheckDataOfferingPageBirthInfoValue(
      getBirthCountry(attributes),
      getBirthDate(attributes),
      getBirthPlace(attributes),
    );
  }

  String getBirthDate(List<T> attributes);

  String getBirthPlace(List<T> attributes);

  String getBirthCountry(List<T> attributes);

  String getSex(List<T> attributes);

  String getNationality(List<T> attributes);

  String getCitizenId(List<T> attributes);

  String getFullAddress(List<T> attributes) {
    final streetName = getStreetName(attributes);
    final houseNumber = getHouseNumber(attributes);
    final postalCode = getPostalCode(attributes);
    final residence = getResidence(attributes);
    return '$streetName $houseNumber, $postalCode $residence';
  }

  String getResidence(List<T> attributes);

  String getPostalCode(List<T> attributes);

  String getStreetName(List<T> attributes);

  String getHouseNumber(List<T> attributes);
}
