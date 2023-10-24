import 'package:collection/collection.dart';

import '../../../wallet_assets.dart';
import '../organization_datasource.dart';

part 'mock_organization_datasource.mocks.dart';

class MockOrganizationDataSource implements OrganizationDataSource {
  @override
  Future<Organization?> read(String organizationId) async {
    return _kOrganizations.firstWhereOrNull((element) => element.id == organizationId);
  }
}
