import '../../../../feature/verification/model/organization.dart';
import '../../../source/organization_datasource.dart';
import '../organization_repository.dart';

class OrganizationRepositoryImpl extends OrganizationRepository {
  final OrganizationDataSource _dataSource;

  OrganizationRepositoryImpl(this._dataSource);

  @override
  Future<Organization?> read(String organizationId) {
    return _dataSource.read(organizationId);
  }
}
