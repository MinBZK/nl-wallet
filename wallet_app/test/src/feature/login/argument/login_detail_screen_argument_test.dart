import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/disclosure/disclose_card_request.dart';
import 'package:wallet/src/feature/login/argument/login_detail_screen_argument.dart';

import '../../../mocks/wallet_mock_data.dart';

void main() {
  test(
    'identical LoginDetailScreenArgument objects are considered equal',
    () {
      final argument = LoginDetailScreenArgument(
        cardRequests: [],
        policy: WalletMockData.policy,
        organization: WalletMockData.organization,
        sharedDataWithOrganizationBefore: false,
      );
      final identicalArgument = LoginDetailScreenArgument(
        cardRequests: [],
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
        cardRequests: [],
        policy: WalletMockData.policy,
        organization: WalletMockData.organization,
        sharedDataWithOrganizationBefore: false,
      );
      final differentArgument = LoginDetailScreenArgument(
        cardRequests: [DiscloseCardRequest.fromCard(WalletMockData.card)],
        policy: WalletMockData.policy,
        organization: WalletMockData.organization,
        sharedDataWithOrganizationBefore: false,
      );
      expect(differentArgument, isNot(argument));
    },
  );
}
