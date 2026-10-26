import 'dart:async';

import '../../wallet_core/error/core_error.dart';

/// A listener for application-level events.
abstract class AppEventListener {
  /// Called when the wallet is unlocked.
  FutureOr<void> onWalletUnlocked() {}

  /// Called when the wallet is locked.
  FutureOr<void> onWalletLocked() {}

  /// Called whenever the dashboard is shown.
  FutureOr<void> onDashboardShown() {}

  /// Called whenever the dashboard is no longer shown, e.g. because another screen covers it or the wallet locked.
  FutureOr<void> onDashboardHidden() {}

  /// Called whenever the core exposes any error
  FutureOr<void> onCoreError(CoreError error) {}
}
