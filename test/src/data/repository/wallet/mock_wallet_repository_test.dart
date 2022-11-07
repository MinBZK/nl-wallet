import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/data/repository/wallet/mock_wallet_repository.dart';
import 'package:wallet/src/data/repository/wallet/wallet_repository.dart';
import 'package:wallet/src/wallet_constants.dart';

void main() {
  late WalletRepository walletRepository;

  setUp(() {
    walletRepository = MockWalletRepository();
  });

  group('Wallet Management', () {
    test('mock wallet is initialized by default', () async {
      expect(await walletRepository.isInitializedStream.first, true);
    });
    test('wallet should not be initialized after destruction', () async {
      await walletRepository.destroyWallet();
      expect(await walletRepository.isInitializedStream.first, false);
    });
    test('wallet should be initialized after creation', () async {
      //Since it default to initialized, we destroy it to make sure it's in the correct state
      await walletRepository.destroyWallet();
      await walletRepository.createWallet(kMockPin);
      expect(await walletRepository.isInitializedStream.first, true);
    });
    test('wallet is locked by default', () async {
      expect(await walletRepository.isLockedStream.first, true);
    });
    test('wallet is unlocked when providing correct pin', () async {
      walletRepository.unlockWallet(kMockPin);
      expect(await walletRepository.isLockedStream.first, false);
    });
    test('wallet is locked after call to lockWallet', () async {
      walletRepository.unlockWallet(kMockPin);
      walletRepository.lockWallet();
      expect(await walletRepository.isLockedStream.first, true);
    });
  });

  group('Pin Attempts', () {
    test('wallet is initialized with 3 available pin attempts', () async {
      expect(walletRepository.leftoverUnlockAttempts, 3);
    });
    test('leftover attempts decrement as user tries to unlock with invalid pin', () async {
      expect(walletRepository.leftoverUnlockAttempts, 3);
      walletRepository.unlockWallet('invalid');
      expect(walletRepository.leftoverUnlockAttempts, 2);
      walletRepository.unlockWallet('invalid');
      expect(walletRepository.leftoverUnlockAttempts, 1);
    });
    test('wallet is destroyed after too many invalid lock attempts', () async {
      walletRepository.unlockWallet('invalid');
      expect(await walletRepository.isInitializedStream.first, true);
      walletRepository.unlockWallet('invalid');
      expect(await walletRepository.isInitializedStream.first, true);
      walletRepository.unlockWallet('invalid');
      expect(await walletRepository.isInitializedStream.first, false);
    });
    test('attempts are reset after creating a new wallet', () async {
      walletRepository.unlockWallet('invalid');
      walletRepository.unlockWallet('invalid');
      expect(walletRepository.leftoverUnlockAttempts, 1);
      await walletRepository.destroyWallet();
      await walletRepository.createWallet(kMockPin);
      expect(walletRepository.leftoverUnlockAttempts, 3);
    });
    test('attempts are reset after successfully unlocking a wallet', () async {
      walletRepository.unlockWallet('invalid');
      walletRepository.unlockWallet('invalid');
      expect(walletRepository.leftoverUnlockAttempts, 1);
      walletRepository.unlockWallet(kMockPin);
      expect(walletRepository.leftoverUnlockAttempts, 3);
    });
  });
}
