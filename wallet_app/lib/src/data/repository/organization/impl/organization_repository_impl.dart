import '../../../../feature/verification/model/organization.dart';
import '../../../source/organization_datasource.dart';
import '../organization_repository.dart';

class OrganizationRepositoryImpl extends OrganizationRepository {
  final OrganizationDataSource dataSource;

  OrganizationRepositoryImpl(this.dataSource);

  @override
  Future<Organization?> read(String organizationId) {
    return dataSource.read(organizationId);
  }
}
