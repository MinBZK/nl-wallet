import '../../../data/repository/organization/organization_repository.dart';
import '../../../domain/model/localized_text.dart';
import '../../../wallet_core/wallet_core.dart';
import '../../extension/string_extension.dart';
import '../mapper.dart';

class RelyingPartyMapper extends Mapper<RelyingParty, Organization> {
  final Mapper<List<LocalizedString>, LocalizedText> _localizedStringMapper;

  RelyingPartyMapper(this._localizedStringMapper);

  @override
  Organization map(RelyingParty input) => Organization(
        id: 'id (missing from core)',
        legalName: _localizedStringMapper.map(input.legalName),
        displayName: _localizedStringMapper.map(input.displayName),
        description: _localizedStringMapper.map(input.description),
        logoUrl: '',
        type: 'type (missing from core)'.untranslated,
        kvk: input.kvk,
        //TODO: Needs mapping from ISO-3166-1 alpha-2 to a localized string. PVW-1656
        country: input.countryCode == null ? null : input.countryCode!.untranslated,
        city: input.city == null ? null : _localizedStringMapper.map(input.city!),
        webUrl: input.webUrl,
      );
}
