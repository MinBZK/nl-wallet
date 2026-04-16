import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/util/mapper/event/wallet_event_status_color_mapper.dart';

import '../../../mocks/wallet_mock_data.dart';

void main() {
  late WalletEventStatusColorMapper mapper;

  setUp(() {
    mapper = WalletEventStatusColorMapper();
  });

  group('useErrorColor', () {
    test('DeletionEvent does not use error color', () {
      expect(mapper.useErrorColor(WalletMockData.deletionEvent), isFalse);
    });

    test('Successful DisclosureEvent does not use error color', () {
      expect(mapper.useErrorColor(WalletMockData.disclosureEvent), isFalse);
    });

    test('Cancelled DisclosureEvent uses error color', () {
      expect(mapper.useErrorColor(WalletMockData.cancelledDisclosureEvent), isTrue);
    });

    test('Failed DisclosureEvent uses error color', () {
      expect(mapper.useErrorColor(WalletMockData.failedDisclosureEvent), isTrue);
    });

    test('Successful IssuanceEvent does not use error color', () {
      expect(mapper.useErrorColor(WalletMockData.issuanceEvent), isFalse);
    });

    test('Renewed IssuanceEvent does not use error color', () {
      expect(mapper.useErrorColor(WalletMockData.issuanceEventCardRenewed), isFalse);
    });

    test('Expired IssuanceEvent uses error color', () {
      expect(mapper.useErrorColor(WalletMockData.issuanceEventCardStatusExpired), isTrue);
    });

    test('Revoked IssuanceEvent uses error color', () {
      expect(mapper.useErrorColor(WalletMockData.issuanceEventCardStatusRevoked), isTrue);
    });

    test('Corrupted IssuanceEvent uses error color', () {
      expect(mapper.useErrorColor(WalletMockData.issuanceEventCardStatusCorrupted), isTrue);
    });

    test('SignEvent does not use error color', () {
      expect(mapper.useErrorColor(WalletMockData.signEvent), isFalse);
    });
  });
}
