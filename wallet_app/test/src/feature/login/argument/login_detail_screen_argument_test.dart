import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/login/argument/login_detail_screen_argument.dart';

import '../../../mocks/wallet_mock_data.dart';

void main() {
  test(
    'identical LoginDetailScreenArgument objects are considered equal',
    () {
      final argument = LoginDetailScreenArgument(
        requestedAttributes: const {},
        policy: WalletMockData.policy,
        organization: WalletMockData.organization,
        sharedDataWithOrganizationBefore: false,
      );
      final identicalArgument = LoginDetailScreenArgument(
        requestedAttributes: const {},
        policy: WalletMockData.policy,
        organization: WalletMockData.organization,
        sharedDataWithOrganizationBefore: false,
      );
      expect(identicalArgument, argument);
    },
  );

  test(
    'different LoginDetailScreenArgument objects are considered not equal',
    () {
      final argument = LoginDetailScreenArgument(
        requestedAttributes: const {},
        policy: WalletMockData.policy,
        organization: WalletMockData.organization,
        sharedDataWithOrganizationBefore: false,
      );
      final differentArgument = LoginDetailScreenArgument(
        requestedAttributes: {WalletMockData.card: const []},
        policy: WalletMockData.policy,
        organization: WalletMockData.organization,
        sharedDataWithOrganizationBefore: false,
      );
      expect(differentArgument, isNot(argument));
    },
  );
}
