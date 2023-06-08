import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/data/mapper/pin/pin_validation_error_mapper.dart';
import 'package:wallet/src/data/repository/wallet/core/core_wallet_repository.dart';

import '../../../../mocks/wallet_mocks.dart';

const _kValidPin = '112233';

void main() {
  late CoreWalletRepository repo;
  final MockTypedWalletCore core = Mocks.create<MockTypedWalletCore>();
  late BehaviorSubject<bool> mockLockedStream;

  setUp(() {
    var registered = false;
    mockLockedStream = BehaviorSubject.seeded(true);
    when(core.isLocked).thenAnswer((_) => mockLockedStream);
    when(core.isRegistered()).thenAnswer((_) async => registered);
    when(core.register(any)).thenAnswer((_) async => registered = true);
    when(core.unlockWallet(any)).thenAnswer((realInvocation) async {
      final pinIsValid = realInvocation.positionalArguments.first == _kValidPin;
      mockLockedStream.add(!pinIsValid);
    });
    when(core.lockWallet()).thenAnswer((realInvocation) async => mockLockedStream.add(true));
    repo = CoreWalletRepository(core, PinValidationErrorMapper());
  });

  group('locked state', () {
    test('locked defaults to true', () async {
      expect((await repo.isLockedStream.first), true);
    });

    test('locked state is updated when unlocked', () async {
      // Setup to make sure wallet is registered and can be unlocked
      await repo.createWallet(_kValidPin);
      repo.lockWallet();

      expectLater(repo.isLockedStream, emitsInOrder([true, false]));
      await repo.unlockWallet(_kValidPin);
    });

    test('locked state is updated when locked', () async {
      // Setup to make sure wallet is registered and can is unlocked
      await repo.createWallet(_kValidPin);

      expectLater(repo.isLockedStream, emitsInOrder([false, true]));
      repo.lockWallet();
    });

    test('locked state is not updated when incorrect pin is provided', () async {
      expectLater(repo.isLockedStream, emitsInOrder([true]));
      await repo.createWallet('invalid');
    });

    test('unlocking is not possible when not registered', () async {
      expect(() => repo.unlockWallet(_kValidPin), throwsUnsupportedError);
    });
  });

  group('registered state', () {
    test('registered state defaults to false', () async {
      final registered = await repo.isRegistered();
      expect(registered, false);
    });
    test('registered is true after wallet creation', () async {
      await repo.createWallet(_kValidPin);

      final registered = await repo.isRegistered();
      expect(registered, true);
    });
  });

  group('wallet creation', () {
    test('wallet is unlocked after registration', () async {
      expectLater(repo.isLockedStream, emitsInOrder([true, false]));
      repo.createWallet(_kValidPin);
    });
  });

  group('unimplemented', () {
    // This group makes sure that, once features are implemented, we are reminded to update the tests.
    test('destroyWallet', () async {
      expect(() => repo.destroyWallet(), throwsUnimplementedError);
    });
    test('leftoverPinAttempts', () async {
      expect(repo.leftoverPinAttempts, 999, reason: 'When this is actually implemented it should never be 999');
    });
    test('confirmTransaction', () async {
      expect(() => repo.confirmTransaction(_kValidPin), throwsUnimplementedError);
    });
  });
}
