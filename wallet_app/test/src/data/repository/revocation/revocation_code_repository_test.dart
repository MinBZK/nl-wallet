import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/revocation/impl/revocation_code_repository_impl.dart';
import 'package:wallet_core/core.dart';

import '../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockRevocationCodeStore store;
  late MockTypedWalletCore core;
  late RevocationRepositoryImpl repository;

  setUp(() {
    store = MockRevocationCodeStore();
    core = MockTypedWalletCore();
    repository = RevocationRepositoryImpl(core, store);
  });

  group('RevocationRepository', () {
    test('getRevocationCodeSaved calls store', () async {
      when(store.getRevocationCodeSavedFlag()).thenAnswer((_) async => true);

      final result = await repository.getRevocationCodeSaved();

      expect(result, true);
      verify(store.getRevocationCodeSavedFlag()).called(1);
    });

    test('setRevocationCodeSaved calls store', () async {
      when(store.setRevocationCodeSavedFlag(saved: true)).thenAnswer((_) async {});

      await repository.setRevocationCodeSaved(saved: true);

      verify(store.setRevocationCodeSavedFlag(saved: true)).called(1);
    });

    test('getRegistrationRevocationCode calls core', () async {
      const code = '123456';
      when(core.getRegistrationRevocationCode()).thenAnswer((_) async => code);

      final result = await repository.getRegistrationRevocationCode();

      expect(result, code);
      verify(core.getRegistrationRevocationCode()).called(1);
    });

    test('getRevocationCode calls core and returns code on success', () async {
      const pin = '1234';
      const code = '654321';
      when(core.getRevocationCode(pin)).thenAnswer((_) async => const RevocationCodeResult.ok(revocationCode: code));

      final result = await repository.getRevocationCode(pin);

      expect(result, code);
      verify(core.getRevocationCode(pin)).called(1);
    });

    test('getRevocationCode calls core and throws on error', () async {
      const pin = '1234';
      const error = WalletInstructionError.incorrectPin(attemptsLeftInRound: 2, isFinalRound: false);
      when(
        core.getRevocationCode(pin),
      ).thenAnswer((_) async => const RevocationCodeResult.instructionError(error: error));

      expect(() => repository.getRevocationCode(pin), throwsA(error));
      verify(core.getRevocationCode(pin)).called(1);
    });
  });
}
