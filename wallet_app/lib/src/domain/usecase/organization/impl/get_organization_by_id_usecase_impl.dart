import '../../../../data/source/organization_datasource.dart';
import '../../../../feature/verification/model/organization.dart';
import '../get_organization_by_id_usecase.dart';

class GetOrganizationByIdUseCaseImpl implements GetOrganizationByIdUseCase {
  final OrganizationDataSource _dataSource;

  GetOrganizationByIdUseCaseImpl(this._dataSource);

  @override
  Future<Organization?> invoke(String organizationId) => _dataSource.read(organizationId);
}
