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
    test('mock wallet is not initialized by default', () async {
      expect(await walletRepository.isInitializedStream.first, false);
    });
    test('wallet should be initialized after creation', () async {
      await walletRepository.createWallet(kMockPin);
      expect(await walletRepository.isInitializedStream.first, true);
    });
    test('destroy wallet should throw when it was not initialized', () async {
      expect(() async => await walletRepository.destroyWallet(), throwsA(isA<UnsupportedError>()));
    });
    test('wallet should not be initialized after destruction', () async {
      await walletRepository.createWallet(kMockPin);
      await walletRepository.destroyWallet();
      expect(await walletRepository.isInitializedStream.first, false);
    });
    test('wallet is locked by default', () async {
      expect(await walletRepository.isLockedStream.first, true);
    });
    test('wallet is unlocked when providing correct pin', () async {
      await walletRepository.createWallet(kMockPin);
      expect(await walletRepository.isLockedStream.first, true);
      walletRepository.unlockWallet(kMockPin);
      expect(await walletRepository.isLockedStream.first, false);
    });
    test('wallet is locked after call to lockWallet', () async {
      await walletRepository.createWallet(kMockPin);
      walletRepository.unlockWallet(kMockPin);
      walletRepository.lockWallet();
      expect(await walletRepository.isLockedStream.first, true);
    });
  });

  group('Pin Attempts', () {
    test('wallet is initialized with 3 available pin attempts', () async {
      await walletRepository.createWallet(kMockPin);
      expect(walletRepository.leftoverPinAttempts, 3);
    });
    test('leftover attempts decrement as user tries to unlock with invalid pin', () async {
      await walletRepository.createWallet(kMockPin);
      expect(walletRepository.leftoverPinAttempts, 3);
      walletRepository.unlockWallet('invalid');
      expect(walletRepository.leftoverPinAttempts, 2);
      walletRepository.unlockWallet('invalid');
      expect(walletRepository.leftoverPinAttempts, 1);
    });
    test('wallet is destroyed after too many invalid lock attempts', () async {
      await walletRepository.createWallet(kMockPin);
      walletRepository.unlockWallet('invalid');
      expect(await walletRepository.isInitializedStream.first, true);
      walletRepository.unlockWallet('invalid');
      expect(await walletRepository.isInitializedStream.first, true);
      walletRepository.unlockWallet('invalid');
      expect(await walletRepository.isInitializedStream.first, false);
    });
    test('attempts are reset after creating a new wallet', () async {
      await walletRepository.createWallet(kMockPin);
      walletRepository.unlockWallet('invalid');
      walletRepository.unlockWallet('invalid');
      expect(walletRepository.leftoverPinAttempts, 1);
      await walletRepository.destroyWallet();
      await walletRepository.createWallet(kMockPin);
      expect(walletRepository.leftoverPinAttempts, 3);
    });
    test('attempts are reset after successfully unlocking a wallet', () async {
      await walletRepository.createWallet(kMockPin);
      walletRepository.unlockWallet('invalid');
      walletRepository.unlockWallet('invalid');
      expect(walletRepository.leftoverPinAttempts, 1);
      walletRepository.unlockWallet(kMockPin);
      expect(walletRepository.leftoverPinAttempts, 3);
    });
  });

  group('Confirmation pin attempts', () {
    test('leftover attempts decrement as user tries to confirm with invalid pin', () async {
      await walletRepository.createWallet(kMockPin);
      walletRepository.unlockWallet(kMockPin); //Make sure wallet is unlocked before confirmation
      expect(walletRepository.leftoverPinAttempts, 3);
      walletRepository.confirmTransaction('invalid');
      expect(walletRepository.leftoverPinAttempts, 2);
      walletRepository.confirmTransaction('invalid');
      expect(walletRepository.leftoverPinAttempts, 1);
    });
    test('wallet is destroyed after too many invalid confirmation attempts', () async {
      await walletRepository.createWallet(kMockPin);
      walletRepository.unlockWallet(kMockPin); //Make sure wallet is unlocked before confirmation
      walletRepository.confirmTransaction('invalid');
      expect(await walletRepository.isInitializedStream.first, true);
      walletRepository.confirmTransaction('invalid');
      expect(await walletRepository.isInitializedStream.first, true);
      walletRepository.confirmTransaction('invalid');
      expect(await walletRepository.isInitializedStream.first, false);
    });
    test('attempts are reset after successfully confirming a transaction', () async {
      await walletRepository.createWallet(kMockPin);
      walletRepository.unlockWallet(kMockPin); //Make sure wallet is unlocked before confirmation
      walletRepository.confirmTransaction('invalid');
      walletRepository.confirmTransaction('invalid');
      expect(walletRepository.leftoverPinAttempts, 1);
      walletRepository.confirmTransaction(kMockPin);
      expect(walletRepository.leftoverPinAttempts, 3);
    });
  });
}
