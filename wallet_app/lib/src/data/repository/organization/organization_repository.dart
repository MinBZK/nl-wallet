import '../../../domain/model/organization.dart';

export '../../../domain/model/organization.dart';

abstract class OrganizationRepository {
  Future<Organization?> read(String organizationId);
}
