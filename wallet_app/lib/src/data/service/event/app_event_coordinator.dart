// ignore_for_file: avoid_function_literals_in_foreach_calls

import 'dart:async';

import '../../../domain/app_event/app_event_listener.dart';
import '../../repository/wallet/wallet_repository.dart';

/// Coordinates and broadcasts application events.
///
/// Upon initialization, it subscribes to the [WalletRepository.isLockedStream]
/// to automatically trigger [onWalletLocked] and [onWalletUnlocked] events.
class AppEventCoordinator implements AppEventListener {
  final List<AppEventListener> _listeners;
  late StreamSubscription<bool> _onLockChangedSubscription;

  AppEventCoordinator(WalletRepository walletRepository, this._listeners) {
    _onLockChangedSubscription = walletRepository.isLockedStream
        .skipWhile((locked) => locked /* App starts out locked, we ignore events until the first unlock */)
        .listen(_onLockChanged);
  }

  void addListener(AppEventListener listener) => _listeners.add(listener);

  void removeListener(AppEventListener listener) {
    final removed = _listeners.remove(listener);
    assert(removed, 'Could not remove listener: $listener');
  }

  @override
  void onDashboardShown() => _listeners.forEach((it) => it.onDashboardShown());

  @override
  void onWalletUnlocked() => _listeners.forEach((it) => it.onWalletUnlocked());

  @override
  void onWalletLocked() => _listeners.forEach((it) => it.onWalletLocked());

  void _onLockChanged(bool isLocked) {
    if (isLocked) {
      onWalletLocked();
    } else {
      onWalletUnlocked();
    }
  }

  void dispose() {
    _listeners.clear();
    _onLockChangedSubscription.cancel();
  }
}
