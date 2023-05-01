import '../../../feature/verification/model/organization.dart';

abstract class OrganizationRepository {
  Future<Organization?> read(String organizationId);
}
