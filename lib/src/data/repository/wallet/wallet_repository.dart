abstract class WalletRepository {
  Future<bool> isWalletInitialized();

  void unlockWallet(String pin);

  void lockWallet();

  Stream<bool> get isLockedStream;
}
