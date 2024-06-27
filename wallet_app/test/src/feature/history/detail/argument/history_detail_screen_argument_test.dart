import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/feature/history/detail/argument/history_detail_screen_argument.dart';

import '../../../../mocks/wallet_mock_data.dart';

void main() {
  test(
    'serialize to and from Map<> yields identical object',
    () {
      final expected = HistoryDetailScreenArgument(
        walletEvent: WalletEvent.issuance(
          dateTime: DateTime(2024),
          card: WalletMockData.card,
          status: EventStatus.success,
        ),
      );
      final serialized = expected.toMap();
      final result = HistoryDetailScreenArgument.fromMap(serialized);
      expect(result, expected);
    },
  );
}
