import 'package:rxdart/rxdart.dart';

import '../../../environment.dart';
import '../../feature/lock/auto_lock_observer.dart';

/// A service to help manage the app's auto lock behaviour.
/// Actual autolocking happens through [AutoLockObserver], as that is where user
/// interaction is tracked and the UI can respond accordingly.
class AutoLockService {
  final Subject<void> _activityStream;
  final BehaviorSubject<bool> _autoLockEnabledStream;

  AutoLockService({
    Subject<void>? activityStream,
    BehaviorSubject<bool>? autoLockStream,
  }) : _activityStream = activityStream ?? PublishSubject<void>(),
       _autoLockEnabledStream = autoLockStream ?? BehaviorSubject.seeded(!Environment.disableAutoLock);

  /// Notifies listeners to reset the idle timeout countdown.
  void resetIdleTimeout() => _activityStream.add(null);

  /// A stream that emits an event whenever the idle timeout has been reset, indicating user activity.
  Stream<void> get activityStream => _activityStream.stream;

  /// Enables or disables the auto-lock feature.
  void setAutoLock({required bool enabled}) {
    if (Environment.disableAutoLock) return; // Globally disabled
    if (_autoLockEnabledStream.value == enabled) return; // No change
    _autoLockEnabledStream.add(enabled);
    if (_autoLockEnabledStream.value) resetIdleTimeout(); // Make sure timer restarts
  }

  /// Returns the current state of the auto-lock feature.
  bool get autoLockEnabled => _autoLockEnabledStream.value;

  /// A stream that emits the current state of the auto-lock feature (enabled/disabled).
  Stream<bool> get autoLockStream => _autoLockEnabledStream.stream;

  void dispose() {
    _activityStream.close();
    _autoLockEnabledStream.close();
  }
}
