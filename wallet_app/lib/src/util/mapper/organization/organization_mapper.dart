import 'package:wallet_core/core.dart' as core show Organization, PersonType;
import 'package:wallet_core/core.dart' hide Organization, PersonType;

import '../../../domain/model/localized_text.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/person_type.dart';
import '../../extension/locale_extension.dart';
import '../mapper.dart';

class OrganizationMapper extends Mapper<core.Organization, Organization> {
  @override
  Organization map(core.Organization input) => Organization(
    id: input.hashCode.toString(),
    legalName: input.legalName,
    displayName: input.displayName,
    publicBody: input.publicBody,
    personType: _mapPersonType(input.personType),
    description: _mapDescription(input.serviceDescription),
    organizationId: input.identifier,
    countryCode: input.countryCode,
    webUri: input.webUrl,
    supportUri: _mapSupportUri(input.supportUri),
    privacyPolicyUri: input.privacyPolicyUrl,
  );

  PersonType _mapPersonType(core.PersonType personType) => switch (personType) {
    core.PersonType.LegalPerson => PersonType.legalPerson,
    core.PersonType.NaturalPerson => PersonType.naturalPerson,
  };

  /// The registration certificate allows a bare email address as support_uri, which can't be launched as is.
  String _mapSupportUri(String supportUri) {
    final trimmed = supportUri.trim();
    if (Uri.tryParse(trimmed)?.hasScheme ?? false) return trimmed;
    if (trimmed.contains('@')) return Uri(scheme: 'mailto', path: trimmed).toString();
    throw const FormatException('Support URI must be a URI or email address');
  }

  LocalizedText? _mapDescription(List<ServiceDescription> descriptions) {
    final result = <Locale, String>{};
    for (final translation in descriptions.expand((description) => description.translations)) {
      final locale = LocaleExtension.parseLocale(translation.language);
      result.update(locale, (value) => '$value\n${translation.value}', ifAbsent: () => translation.value);
    }
    return result.isEmpty ? null : result;
  }
}
