import '../../../../data/repository/organization/organization_repository.dart';
import '../get_organization_by_id_usecase.dart';

class GetOrganizationByIdUseCaseImpl implements GetOrganizationByIdUseCase {
  final OrganizationRepository _organizationRepository;

  GetOrganizationByIdUseCaseImpl(this._organizationRepository);

  @override
  Future<Organization?> invoke(String organizationId) {
    return _organizationRepository.read(organizationId);
  }
}
