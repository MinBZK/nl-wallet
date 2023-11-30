import 'package:wallet_core/core.dart' as core;
import 'package:wallet_mock/mock.dart';

import '../../../../util/mapper/mapper.dart';
import '../organization_repository.dart';

class OrganizationRepositoryImpl extends OrganizationRepository {
  final WalletCoreForIssuance _core;
  final Mapper<core.Organization, Organization> _organizationMapper;

  OrganizationRepositoryImpl(this._core, this._organizationMapper);

  @override
  Future<Organization?> read(String organizationId) {
    throw UnsupportedError('mock leftover');
  }

  @override
  Future<Organization?> findIssuer(String docType) async {
    final coreOrganization = await _core.getIssuer(docType);
    return _organizationMapper.map(coreOrganization);
  }
}
