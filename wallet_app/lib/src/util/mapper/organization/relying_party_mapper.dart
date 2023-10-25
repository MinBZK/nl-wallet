import '../../../data/repository/organization/organization_repository.dart';
import '../../../wallet_core/wallet_core.dart';
import '../mapper.dart';

class RelyingPartyMapper extends Mapper<RelyingParty, Organization> {
  RelyingPartyMapper();

  @override
  Organization map(RelyingParty input) => Organization(
        id: '',
        name: input.name,
        shortName: input.name,
        category: '',
        description: '',
        logoUrl: '',
      );
}
