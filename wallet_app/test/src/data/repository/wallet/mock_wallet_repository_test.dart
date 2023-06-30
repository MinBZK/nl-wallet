import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/bridge_generated.dart';
import 'package:wallet/src/data/repository/wallet/mock/mock_wallet_repository.dart';
import 'package:wallet/src/data/repository/wallet/wallet_repository.dart';
import 'package:wallet/src/data/source/memory/memory_wallet_datasource.dart';
import 'package:wallet/src/wallet_constants.dart';

void main() {
  /// Future todo; add mocking framework and replace with mocked [WalletDataSource]
  final walletDataSource = MemoryWalletDataSource();

  late WalletRepository walletRepository;

  setUp(() {
    walletRepository = MockWalletRepository(walletDataSource);
  });

  group('Wallet Management', () {
    test('mock wallet is not initialized by default', () async {
      expect(await walletRepository.isRegistered(), false);
    });
    test('wallet should be initialized after creation', () async {
      await walletRepository.createWallet(kMockPin);
      expect(await walletRepository.isRegistered(), true);
    });
    test('destroy wallet should throw when it was not initialized', () async {
      expect(() async => await walletRepository.destroyWallet(), throwsA(isA<UnsupportedError>()));
    });
    test('wallet should not be initialized after destruction', () async {
      await walletRepository.createWallet(kMockPin);
      await walletRepository.destroyWallet();
      expect(await walletRepository.isRegistered(), false);
    });
    test('wallet is locked by default', () async {
      expect(await walletRepository.isLockedStream.first, true);
    });
    test('wallet is unlocked after creation', () async {
      await walletRepository.createWallet(kMockPin);
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
    test('leftover attempts decrement as user tries to unlock with invalid pin', () async {
      await walletRepository.createWallet(kMockPin);
      var result = await walletRepository.unlockWallet('invalid') as WalletUnlockResult_IncorrectPin;
      expect(result.leftoverAttempts, 3);
      result = await walletRepository.unlockWallet('invalid') as WalletUnlockResult_IncorrectPin;
      expect(result.leftoverAttempts, 2);
      result = await walletRepository.unlockWallet('invalid') as WalletUnlockResult_IncorrectPin;
      expect(result.leftoverAttempts, 1);
    });
    test('attempts are reset after creating a new wallet', () async {
      await walletRepository.createWallet(kMockPin);
      var result = await walletRepository.unlockWallet('invalid') as WalletUnlockResult_IncorrectPin;
      expect(result.leftoverAttempts, 3);
      await walletRepository.destroyWallet();
      await walletRepository.createWallet(kMockPin);
      result = await walletRepository.unlockWallet('invalid') as WalletUnlockResult_IncorrectPin;
      expect(result.leftoverAttempts, 3);
    });
    test('attempts are reset after successfully unlocking a wallet', () async {
      await walletRepository.createWallet(kMockPin);
      await walletRepository.unlockWallet('invalid') as WalletUnlockResult_IncorrectPin;
      var result = await walletRepository.unlockWallet('invalid') as WalletUnlockResult_IncorrectPin;
      expect(result.leftoverAttempts, 2);
      walletRepository.unlockWallet(kMockPin);
      walletRepository.lockWallet();
      result = await walletRepository.unlockWallet('invalid') as WalletUnlockResult_IncorrectPin;
      expect(result.leftoverAttempts, 3);
    });
  });

  group('Confirmation pin attempts', () {
    test('wallet is destroyed after too many invalid confirmation attempts', () async {
      await walletRepository.createWallet(kMockPin);
      await walletRepository.unlockWallet(kMockPin); //Make sure wallet is unlocked before confirmation
      expect(await walletRepository.isRegistered(), true);
      for (var _ in Iterable.generate(kMaxUnlockAttempts)) {
        await walletRepository.confirmTransaction('invalid');
      }
      expect(await walletRepository.isRegistered(), false);
    });
    test('attempts are reset after successfully confirming a transaction', () async {
      await walletRepository.createWallet(kMockPin);
      walletRepository.unlockWallet(kMockPin); //Make sure wallet is unlocked before confirmation
      walletRepository.confirmTransaction('invalid');
      walletRepository.confirmTransaction('invalid');
      walletRepository.confirmTransaction(kMockPin);
      walletRepository.confirmTransaction('invalid');
      walletRepository.confirmTransaction('invalid');
      //Should be blocked now unless the attempts were reset, this is confirmed by the test above
      expect(await walletRepository.isRegistered(), true);
    });
  });
}
