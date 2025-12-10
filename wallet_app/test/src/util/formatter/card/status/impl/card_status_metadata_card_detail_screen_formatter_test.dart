import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/util/formatter/card/status/impl/card_status_metadata_card_detail_screen_formatter.dart';

import '../../../../../mocks/wallet_mock_data.dart';

void main() {
  late CardStatusMetadataCardDetailScreenFormatter formatter;

  setUp(() {
    formatter = CardStatusMetadataCardDetailScreenFormatter();
  });

  group('show', () {
    test('returns true for all card statuses', () {
      for (final status in WalletMockData.cardStatusList) {
        expect(formatter.show(status), true, reason: 'Failed for status: $status');
      }
    });
  });
}
