import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/util/mapper/event/wallet_event_status_icon_mapper.dart';

import '../../../mocks/wallet_mock_data.dart';

void main() {
  late WalletEventStatusIconMapper mapper;

  setUp(() {
    mapper = WalletEventStatusIconMapper();
  });

  group('DeletionEvent', () {
    test('returns null (no error icon for successful deletion)', () {
      expect(mapper.map(WalletMockData.deletionEvent), isNull);
    });
  });

  group('DisclosureEvent', () {
    test('returns null for success', () {
      expect(mapper.map(WalletMockData.disclosureEvent), isNull);
    });

    test('returns block_flipped for cancelled', () {
      expect(mapper.map(WalletMockData.cancelledDisclosureEvent), Icons.block_flipped);
    });

    test('returns error_outline_outlined for error', () {
      expect(mapper.map(WalletMockData.failedDisclosureEvent), Icons.error_outline_outlined);
    });
  });

  group('IssuanceEvent', () {
    test('returns null for cardIssued', () {
      expect(mapper.map(WalletMockData.issuanceEvent), isNull);
    });

    test('returns null for cardRenewed', () {
      expect(mapper.map(WalletMockData.issuanceEventCardRenewed), isNull);
    });

    test('returns event_busy for cardStatusExpired', () {
      expect(mapper.map(WalletMockData.issuanceEventCardStatusExpired), Icons.event_busy);
    });

    test('returns close for cardStatusRevoked', () {
      expect(mapper.map(WalletMockData.issuanceEventCardStatusRevoked), Icons.close);
    });

    test('returns block_flipped for cardStatusCorrupted', () {
      expect(mapper.map(WalletMockData.issuanceEventCardStatusCorrupted), Icons.block_flipped);
    });
  });

  group('SignEvent', () {
    test('returns null for success', () {
      expect(mapper.map(WalletMockData.signEvent), isNull);
    });
  });
}
