import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/util/extension/string_extension.dart';

import '../../../mocks/wallet_mock_data.dart';

void main() {
  test('DisclosureEvent', () {
    final event = WalletEvent.disclosure(
      dateTime: DateTime(2024),
      status: EventStatus.success,
      relyingParty: WalletMockData.organization,
      purpose: 'purpose'.untranslated,
      cards: [WalletMockData.card],
      policy: WalletMockData.policy,
      type: DisclosureType.regular,
    );
    final identicalEvent = WalletEvent.disclosure(
      dateTime: DateTime(2024),
      status: EventStatus.success,
      relyingParty: WalletMockData.organization,
      purpose: 'purpose'.untranslated,
      cards: [WalletMockData.card],
      policy: WalletMockData.policy,
      type: DisclosureType.regular,
    );
    final differentEvent = WalletEvent.disclosure(
      dateTime: DateTime(2024),
      status: EventStatus.success,
      relyingParty: WalletMockData.organization,
      purpose: 'purpose'.untranslated,
      cards: [WalletMockData.card],
      policy: WalletMockData.policy,
      type: DisclosureType.login,
    );
    expect(event, identicalEvent);
    expect(event, isNot(equals(differentEvent)));
  });

  test('IssuanceEvent', () {
    final event = WalletEvent.issuance(
      dateTime: DateTime(2024),
      status: EventStatus.success,
      card: WalletMockData.card,
      renewed: false,
    );
    final identicalEvent = WalletEvent.issuance(
      dateTime: DateTime(2024),
      status: EventStatus.success,
      card: WalletMockData.card,
      renewed: false,
    );
    final differentEvent = WalletEvent.issuance(
      dateTime: DateTime(2024),
      status: EventStatus.cancelled,
      card: WalletMockData.card,
      renewed: false,
    );
    expect(event, identicalEvent);
    expect(event, isNot(equals(differentEvent)));
  });

  test('SignEvent', () {
    final event = WalletEvent.sign(
      dateTime: DateTime(2024),
      status: EventStatus.success,
      relyingParty: WalletMockData.organization,
      policy: WalletMockData.policy,
      document: WalletMockData.document,
    );
    final identicalEvent = WalletEvent.sign(
      dateTime: DateTime(2024),
      status: EventStatus.success,
      relyingParty: WalletMockData.organization,
      policy: WalletMockData.policy,
      document: WalletMockData.document,
    );
    final differentEvent = WalletEvent.sign(
      dateTime: DateTime(2023),
      status: EventStatus.success,
      relyingParty: WalletMockData.organization,
      policy: WalletMockData.policy,
      document: WalletMockData.document,
    );
    expect(event, identicalEvent);
    expect(event, isNot(equals(differentEvent)));
  });
}
