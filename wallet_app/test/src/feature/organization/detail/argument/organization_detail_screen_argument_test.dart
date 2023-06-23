import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/organization/detail/argument/organization_detail_screen_argument.dart';

void main() {
  test(
    'serialize to and from Map<> yields identical object',
    () {
      const expected = OrganizationDetailScreenArgument(title: 'Lorem Ipsum', organizationId: '9-9');
      final serialized = expected.toMap();
      final result = OrganizationDetailScreenArgument.fromMap(serialized);
      expect(result, expected);
    },
  );
}
