/// The requirements that need to be fulfilled before the wallet can navigate
enum NavigationPrerequisite {
  /// The wallet must be unlocked before proceeding.
  walletUnlocked,

  /// The wallet must be initialized before proceeding.
  walletInitialized,

  /// The wallet must be in a ready state (see [WalletState]) before proceeding.
  walletInReadyState,

  /// The wallet must be in the issuance state (see [WalletState]) before proceeding.
  walletInIssuanceState,

  /// The PID must be initialized before proceeding.
  pidInitialized,
}
