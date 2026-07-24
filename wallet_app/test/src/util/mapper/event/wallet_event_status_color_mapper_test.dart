import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/util/mapper/event/wallet_event_status_color_mapper.dart';

import '../../../mocks/wallet_mock_data.dart';

void main() {
  late WalletEventStatusColorMapper mapper;

  setUp(() {
    mapper = WalletEventStatusColorMapper();
  });

  group('useErrorColor', () {
    test('ltc22 ltc23 DeletionEvent does not use error color', () {
      expect(mapper.useErrorColor(WalletMockData.deletionEvent), isFalse);
    });

    test('ltc22 ltc23 Successful DisclosureEvent does not use error color', () {
      expect(mapper.useErrorColor(WalletMockData.disclosureEvent), isFalse);
    });

    test('ltc22 ltc23 Cancelled DisclosureEvent uses error color', () {
      expect(mapper.useErrorColor(WalletMockData.cancelledDisclosureEvent), isTrue);
    });

    test('ltc22 ltc23 Failed DisclosureEvent uses error color', () {
      expect(mapper.useErrorColor(WalletMockData.failedDisclosureEvent), isTrue);
    });

    test('ltc22 ltc23 Successful IssuanceEvent does not use error color', () {
      expect(mapper.useErrorColor(WalletMockData.issuanceEvent), isFalse);
    });

    test('ltc22 ltc23 Renewed IssuanceEvent does not use error color', () {
      expect(mapper.useErrorColor(WalletMockData.issuanceEventCardRenewed), isFalse);
    });

    test('ltc22 ltc23 Expired IssuanceEvent uses error color', () {
      expect(mapper.useErrorColor(WalletMockData.issuanceEventCardStatusExpired), isTrue);
    });

    test('ltc22 ltc23 Revoked IssuanceEvent uses error color', () {
      expect(mapper.useErrorColor(WalletMockData.issuanceEventCardStatusRevoked), isTrue);
    });

    test('ltc22 ltc23 Corrupted IssuanceEvent uses error color', () {
      expect(mapper.useErrorColor(WalletMockData.issuanceEventCardStatusCorrupted), isTrue);
    });

    test('ltc22 ltc23 SignEvent does not use error color', () {
      expect(mapper.useErrorColor(WalletMockData.signEvent), isFalse);
    });
  });
}
