import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/dashboard/argument/dashboard_screen_argument.dart';

import '../../../mocks/wallet_mock_data.dart';

void main() {
  test('ltc24 verify json serialization', () {
    final original = DashboardScreenArgument(cards: [WalletMockData.card]);
    final json = original.toJson();
    final deserialized = DashboardScreenArgument.fromJson(json);
    expect(original, deserialized);
  });
}
