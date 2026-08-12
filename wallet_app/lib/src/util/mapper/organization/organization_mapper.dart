import 'package:wallet_core/core.dart' as core show Organization;
import 'package:wallet_core/core.dart' hide Organization;

import '../../../domain/model/app_image_data.dart';
import '../../../domain/model/localized_text.dart';
import '../../../domain/model/organization.dart';
import '../mapper.dart';

class OrganizationMapper extends Mapper<core.Organization, Organization> {
  final Mapper<List<LocalizedString>, LocalizedText> _localizedStringMapper;
  final Mapper<Image, AppImageData> _imageMapper;

  OrganizationMapper(this._localizedStringMapper, this._imageMapper);

  @override
  Organization map(core.Organization input) => Organization(
    id: input.hashCode.toString(),
    legalName: input.legalName,
    displayName: input.displayName,
    description: input.description.isEmpty ? null : _localizedStringMapper.map(input.description),
    logo: input.image == null ? null : _imageMapper.map(input.image!),
    type: input.category.isEmpty ? null : _localizedStringMapper.map(input.category),
    organizationId: input.identifier,
    countryCode: input.countryCode,
    webUri: input.webUrl,
    supportUri: null, // TODO(Anyone): PVW-6111
    privacyPolicyUri: input.privacyPolicyUrl,
  );
}
