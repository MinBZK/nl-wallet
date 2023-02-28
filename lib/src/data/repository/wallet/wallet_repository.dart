abstract class WalletRepository {
  Stream<bool> get isInitializedStream;

  Stream<bool> get isLockedStream;

  /// Validates the supplied [pin]
  ///
  /// Throws a [PinValidationError] if the pin does
  /// not meet the required standards.
  Future<void> validatePin(String pin);

  Future<void> createWallet(String pin);

  Future<void> destroyWallet();

  void unlockWallet(String pin);

  void lockWallet();

  Future<bool> confirmTransaction(String pin);

  int get leftoverPinAttempts;
}
