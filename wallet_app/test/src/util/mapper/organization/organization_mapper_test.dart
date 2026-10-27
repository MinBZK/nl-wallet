import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/localized_text.dart';
import 'package:wallet/src/domain/model/person_type.dart';
import 'package:wallet/src/util/mapper/organization/organization_mapper.dart';
import 'package:wallet_core/core.dart' as core;

void main() {
  final mapper = OrganizationMapper();

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
          supportUri: 'https://example.com/support',
          personType: core.PersonType.LegalPerson,
        ),
      )
      .description;

  String mapSupportUri(String supportUri) => mapper
      .map(
        core.Organization(
          legalName: 'Issuer',
          displayName: 'Issuer',
          serviceDescription: [],
          identifier: 'issuer',
          countryCode: 'NL',
          supportUri: supportUri,
          personType: core.PersonType.LegalPerson,
        ),
      )
      .supportUri;

  for (final publicBody in [null, true, false]) {
    test('should map publicBody $publicBody', () {
      final organization = mapper.map(
        core.Organization(
          legalName: 'Issuer',
          displayName: 'Issuer',
          serviceDescription: [],
          identifier: 'issuer',
          countryCode: 'NL',
          publicBody: publicBody,
          supportUri: 'https://example.com/support',
          personType: core.PersonType.LegalPerson,
        ),
      );
      expect(organization.publicBody, publicBody);
    });
  }

  for (final (personType, expected) in [
    (core.PersonType.LegalPerson, PersonType.legalPerson),
    (core.PersonType.NaturalPerson, PersonType.naturalPerson),
  ]) {
    test('should map personType $personType', () {
      final organization = mapper.map(
        core.Organization(
          legalName: 'Issuer',
          displayName: 'Issuer',
          serviceDescription: [],
          identifier: 'issuer',
          countryCode: 'NL',
          personType: personType,
          supportUri: 'https://example.com/support',
        ),
      );
      expect(organization.personType, expected);
    });
  }

  test('should keep a support uri that has a scheme', () {
    expect(mapSupportUri('https://example.com/support'), 'https://example.com/support');
    expect(mapSupportUri('mailto:support@example.com'), 'mailto:support@example.com');
  });

  test('should turn a bare email address into a mailto uri', () {
    expect(mapSupportUri('support@example.com'), 'mailto:support@example.com');
  });

  test('should trim surrounding whitespace from the support uri', () {
    expect(mapSupportUri(' https://example.com/support '), 'https://example.com/support');
  });

  test('should encode characters that are not allowed in a mailto path', () {
    expect(mapSupportUri('"help:desk"@example.com'), 'mailto:%22help:desk%22@example.com');
  });

  test('should reject a support uri that is neither a uri nor an email address', () {
    expect(() => mapSupportUri('support.example.com'), throwsFormatException);
    expect(() => mapSupportUri(''), throwsFormatException);
    expect(() => mapSupportUri('   '), throwsFormatException);
  });

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
