import '../../domain/model/organization.dart';

export '../../domain/model/organization.dart';

abstract class OrganizationDataSource {
  Future<Organization?> read(String organizationId);
}
