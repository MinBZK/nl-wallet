import '../../feature/verification/model/organization.dart';

abstract class OrganizationDataSource {
  Future<Organization?> read(String organizationId);
}
