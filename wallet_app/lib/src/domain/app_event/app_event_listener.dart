import 'dart:async';

/// A listener for application-level events.
abstract class AppEventListener {
  /// Called when the wallet is unlocked.
  FutureOr<void> onWalletUnlocked() {}

  /// Called when the wallet is locked.
  FutureOr<void> onWalletLocked() {}

  /// Called whenever the dashboard is shown.
  FutureOr<void> onDashboardShown() {}
}
