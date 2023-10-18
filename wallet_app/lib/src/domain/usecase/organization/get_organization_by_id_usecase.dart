import '../../model/organization.dart';

abstract class GetOrganizationByIdUseCase {
  Future<Organization?> invoke(String organizationId);
}
