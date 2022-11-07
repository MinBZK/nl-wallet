abstract class WalletRepository {
  Stream<bool> get isInitializedStream;

  Stream<bool> get isLockedStream;

  Future<bool> createWallet(String pin);

  Future<void> destroyWallet();

  void unlockWallet(String pin);

  void lockWallet();

  int get leftoverUnlockAttempts;
}
