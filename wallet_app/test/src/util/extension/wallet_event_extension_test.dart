import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/util/extension/wallet_event_extension.dart';

import '../../mocks/wallet_mock_data.dart';

void main() {
  group('relyingPartyOrIssuer', () {
    test('DeletionEvent returns the card issuer', () {
      final event = WalletMockData.deletionEvent;
      expect(event.relyingPartyOrIssuer, event.card.issuer);
    });

    test('DisclosureEvent returns the relying party', () {
      final event = WalletMockData.disclosureEvent;
      expect(event.relyingPartyOrIssuer, event.relyingParty);
    });

    test('IssuanceEvent returns the card issuer', () {
      final event = WalletMockData.issuanceEvent;
      expect(event.relyingPartyOrIssuer, event.card.issuer);
    });

    test('SignEvent returns the relying party', () {
      final event = WalletMockData.signEvent;
      expect(event.relyingPartyOrIssuer, event.relyingParty);
    });
  });

  group('status helpers', () {
    test('DeletionEvent reports wasSuccess', () {
      expect(WalletMockData.deletionEvent.wasSuccess, isTrue);
      expect(WalletMockData.deletionEvent.wasCancelled, isFalse);
      expect(WalletMockData.deletionEvent.wasFailure, isFalse);
    });
  });
}
