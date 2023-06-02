import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/mapper/pin/pin_validation_error_mapper.dart';
import 'package:wallet/src/data/repository/wallet/core/core_wallet_repository.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late CoreWalletRepository repo;
  final MockTypedWalletCore core = Mocks.create<MockTypedWalletCore>();

  setUp(() {
    var registered = false;
    when(core.isRegistered()).thenAnswer((_) async => registered);
    when(core.register(any)).thenAnswer((_) async => registered = true);
    repo = CoreWalletRepository(core, PinValidationErrorMapper());
  });

  group('locked state', () {
    test('locked defaults to true', () async {
      expect((await repo.isLockedStream.first), true);
    });

    test('locked state is updated when unlocked', () async {
      // Setup to make sure wallet is registered and can be unlocked
      await repo.createWallet('valid');
      repo.lockWallet();

      expectLater(repo.isLockedStream, emitsInOrder([true, false]));
      await repo.unlockWallet('valid');
    });

    test('locked state is updated when locked', () async {
      // Setup to make sure wallet is registered and can is unlocked
      await repo.createWallet('valid');

      expectLater(repo.isLockedStream, emitsInOrder([false, true]));
      repo.lockWallet();
    });

    test('unlocking is not possible when not registered', () async {
      expect(() => repo.unlockWallet('valid'), throwsUnsupportedError);
    });
  });

  group('registered state', () {
    test('registered state defaults to false', () async {
      final registered = await repo.isRegistered();
      expect(registered, false);
    });
    test('registered is true after wallet creation', () async {
      await repo.createWallet('valid');

      final registered = await repo.isRegistered();
      expect(registered, true);
    });
  });

  group('wallet creation', () {
    test('wallet is unlocked after registration', () async {
      expectLater(repo.isLockedStream, emitsInOrder([true, false]));
      repo.createWallet('valid');
    });
  });

  group('unimplemented', () {
    // This group makes sure that, once features are implemented, we are reminded to update the tests.
    test('destroyWallet', () async {
      expect(() => repo.destroyWallet(), throwsUnimplementedError);
    });
    test('leftoverPinAttempts', () async {
      expect(() => repo.leftoverPinAttempts, throwsUnimplementedError);
    });
    test('confirmTransaction', () async {
      expect(() => repo.confirmTransaction('valid'), throwsUnimplementedError);
    });
  });
}
