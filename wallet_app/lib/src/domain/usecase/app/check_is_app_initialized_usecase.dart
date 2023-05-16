abstract class IsWalletInitializedUseCase {
  /// Check if the app has been initialized, meaning the
  /// wallet has already been registered with the wallet provider.
  Future<bool> invoke();
}
