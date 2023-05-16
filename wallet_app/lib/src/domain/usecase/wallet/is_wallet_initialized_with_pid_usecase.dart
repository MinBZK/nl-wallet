abstract class IsWalletInitializedWithPidUseCase {
  /// Check if the app has been initialized, AND the PID
  /// has been retrieved from the PID provider.
  Future<bool> invoke();
}
