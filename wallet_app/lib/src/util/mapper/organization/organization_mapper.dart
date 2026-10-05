import 'package:wallet_core/core.dart' as core show Organization;
import 'package:wallet_core/core.dart' hide Organization;

import '../../../domain/model/localized_text.dart';
import '../../../domain/model/organization.dart';
import '../../extension/locale_extension.dart';
import '../mapper.dart';

class OrganizationMapper extends Mapper<core.Organization, Organization> {
  // TODO(Anyone): PVW-6101 Remove unused logo/type fields from the domain model and UI.
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
      final locale = LocaleExtension.parseLocale(translation.language);
      result.update(locale, (value) => '$value\n${translation.value}', ifAbsent: () => translation.value);
    }
    return result.isEmpty ? null : result;
  }
}
