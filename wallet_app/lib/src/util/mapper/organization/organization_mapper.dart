import 'package:wallet_core/core.dart' as core show Organization;
import 'package:wallet_core/core.dart' hide Organization;

import '../../../domain/model/localized_text.dart';
import '../../../domain/model/organization.dart';
import '../mapper.dart';

class OrganizationMapper extends Mapper<core.Organization, Organization> {
  final Mapper<List<LocalizedString>, LocalizedText> _localizedStringMapper;
  OrganizationMapper(this._localizedStringMapper);

  @override
  Organization map(core.Organization input) => Organization(
    id: input.hashCode.toString(),
    legalName: input.legalName,
    displayName: input.displayName,
    description: _mapDescription(input.serviceDescription),
    organizationId: input.identifier,
    countryCode: input.countryCode,
    webUri: input.webUrl,
    supportUri: null, // TODO(Anyone): PVW-6111
    privacyPolicyUri: input.privacyPolicyUrl,
  );

  LocalizedText? _mapDescription(List<ServiceDescription> descriptions) {
    final result = <Locale, String>{};
    for (final translation in descriptions.expand((description) => description.translations)) {
      for (final entry in _localizedStringMapper.map([translation]).entries) {
        result.update(entry.key, (value) => '$value\n${entry.value}', ifAbsent: () => entry.value);
      }
    }
    return result.isEmpty ? null : result;
  }
}
