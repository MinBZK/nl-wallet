import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/util/extension/wallet_event_extension.dart';

import '../../mocks/wallet_mock_data.dart';

void main() {
  group('status helpers', () {
    test('DeletionEvent reports wasSuccess', () {
      expect(WalletMockData.deletionEvent.wasSuccess, isTrue);
      expect(WalletMockData.deletionEvent.wasCancelled, isFalse);
      expect(WalletMockData.deletionEvent.wasFailure, isFalse);
    });
  });
}
