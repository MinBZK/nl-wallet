import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/localized_text.dart';
import 'package:wallet/src/util/mapper/card/attribute/localized_labels_mapper.dart';
import 'package:wallet/src/util/mapper/organization/organization_mapper.dart';
import 'package:wallet_core/core.dart' as core;

void main() {
  final mapper = OrganizationMapper(LocalizedLabelsMapper());

  LocalizedText? mapDescription(List<List<core.LocalizedString>> descriptions) => mapper
      .map(
        core.Organization(
          legalName: 'Issuer',
          displayName: 'Issuer',
          serviceDescription: descriptions
              .map((translations) => core.ServiceDescription(translations: translations))
              .toList(),
          identifier: 'issuer',
          countryCode: 'NL',
        ),
      )
      .description;

  test('should return null when no descriptions are available', () {
    expect(mapDescription([]), isNull);
  });

  test('should preserve the translations of a single description', () {
    expect(
      mapDescription([
        [
          const core.LocalizedString(language: 'en', value: 'Service'),
          const core.LocalizedString(language: 'nl', value: 'Dienst'),
        ],
      ]),
      {const Locale('en'): 'Service', const Locale('nl'): 'Dienst'},
    );
  });

  test('should join descriptions by language in their original order', () {
    expect(
      mapDescription([
        [
          const core.LocalizedString(language: 'en', value: 'First service'),
          const core.LocalizedString(language: 'nl', value: 'Eerste dienst'),
          const core.LocalizedString(language: 'en', value: 'Extra details'),
        ],
        [const core.LocalizedString(language: 'en', value: 'Second service')],
      ]),
      {const Locale('en'): 'First service\nExtra details\nSecond service', const Locale('nl'): 'Eerste dienst'},
    );
  });
}
