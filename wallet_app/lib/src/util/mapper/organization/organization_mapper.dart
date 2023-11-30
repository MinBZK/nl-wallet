import 'package:wallet_core/core.dart' as core show Organization;
import 'package:wallet_core/core.dart' hide Organization;

import '../../../data/repository/organization/organization_repository.dart';
import '../../../domain/model/app_image_data.dart';
import '../../../domain/model/localized_text.dart';
import '../../../wallet_assets.dart';
import '../../extension/string_extension.dart';
import '../mapper.dart';

class OrganizationMapper extends Mapper<core.Organization, Organization> {
  final Mapper<List<LocalizedString>, LocalizedText> _localizedStringMapper;
  final Mapper<Image, AppImageData> _imageMapper;

  OrganizationMapper(this._localizedStringMapper, this._imageMapper);

  @override
  Organization map(core.Organization input) => Organization(
        id: input.hashCode.toString(),
        legalName: _localizedStringMapper.map(input.legalName),
        displayName: _localizedStringMapper.map(input.displayName),
        description: _localizedStringMapper.map(input.description),
        logo: input.image == null
            ? const AppAssetImage(WalletAssets.logo_rp_placeholder)
            : _imageMapper.map(input.image!),
        type: 'type (missing from core)'.untranslated,
        kvk: input.kvk,
        countryCode: input.countryCode,
        city: input.city == null ? null : _localizedStringMapper.map(input.city!),
        webUrl: input.webUrl,
      );
}
