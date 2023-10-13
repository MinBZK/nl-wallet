abstract class ObserveWalletLockedUseCase {
  /// Stream exposes [true] when the wallet is currently locked.
  Stream<bool> invoke();
}
