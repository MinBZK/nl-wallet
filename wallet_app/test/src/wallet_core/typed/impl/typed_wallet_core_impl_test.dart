import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/wallet_core/error/core_error_mapper.dart';
import 'package:wallet/src/wallet_core/typed/impl/typed_wallet_core_impl.dart';
import 'package:wallet/src/wallet_core/typed/typed_wallet_core.dart';
import 'package:wallet/src/wallet_core/wallet_core.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  final WalletCore core = Mocks.create();

  late TypedWalletCore typedWalletCore;

  setUp(() {
    typedWalletCore = TypedWalletCoreImpl(core, CoreErrorMapper());

    /// Setup default initialization mock
    when(core.isInitialized()).thenAnswer((realInvocation) async => false);
    when(core.init()).thenAnswer((realInvocation) async => true);
  });

  group('Observe Cards', () {
    test('observeCards should fetch cards through WalletCore', () {
      List<Card> mockCards = [
        const Card(id: 0, docType: 'pid_id', issuer: 'issuer', attributes: []),
        const Card(id: 0, docType: 'pid_address', issuer: 'issuer', attributes: []),
      ];
      when(core.setCardsStream()).thenAnswer((realInvocation) => Stream.value(mockCards));
      expect(typedWalletCore.observeCards(), emits(hasLength(2)));
    });

    test('observeCards should emit a new value when WalletCore exposes new cards', () {
      List<Card> initialCards = [const Card(id: 0, docType: 'pid_id', issuer: 'issuer', attributes: [])];
      List<Card> updatedCards = [
        const Card(id: 0, docType: 'pid_id', issuer: 'issuer', attributes: []),
        const Card(id: 0, docType: 'pid_address', issuer: 'issuer', attributes: []),
      ];
      when(core.setCardsStream()).thenAnswer((realInvocation) => Stream.fromIterable([initialCards, updatedCards]));

      expect(
        typedWalletCore.observeCards(),
        emitsInOrder(
          [
            hasLength(initialCards.length),
            hasLength(updatedCards.length),
          ],
        ),
      );
    });
  });
}
