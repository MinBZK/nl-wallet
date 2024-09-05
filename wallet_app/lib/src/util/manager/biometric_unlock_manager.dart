import 'dart:ui';

import 'package:rxdart/rxdart.dart';

import '../../data/repository/wallet/wallet_repository.dart';
import '../../data/service/app_lifecycle_service.dart';
import '../../domain/usecase/biometrics/is_biometric_login_enabled_usecase.dart';

/// Manages the [shouldTriggerUnlock] state by observing the lifecycle and app
/// lock state. The goal is to show the biometric prompt to unlock the app when:
/// 1) The app is first opened
/// 2) The app is brought back into the foreground while it's in the locked state
/// Though this sounds trivial, we need to manage some logic in this class to avoid
/// showing the biometric prompt when:
/// 1) The users locks the app manually (e.g. from the settings)
/// 2) The app locks while the device is idle (but in the foreground)
/// 3) The user fails to unlock the app with biometrics (and e.g. the PinScreen is rebuild)
class BiometricUnlockManager {
  final AppLifecycleService _appLifecycleService;
  final WalletRepository _walletRepository;
  final IsBiometricLoginEnabledUseCase _isBiometricLoginEnabledUseCase;

  bool _shouldTriggerUnlock = true;

  bool get shouldTriggerUnlock => _shouldTriggerUnlock;

  bool getAndSetShouldTriggerUnlock({required bool updatedValue}) {
    final shouldTrigger = _shouldTriggerUnlock;
    _shouldTriggerUnlock = updatedValue;
    return shouldTrigger;
  }

  BiometricUnlockManager(this._appLifecycleService, this._walletRepository, this._isBiometricLoginEnabledUseCase) {
    _appLifecycleService.observe().listen(_onStateChanged);
    _walletRepository.isLockedStream.listen(_onLockChanged);
  }

  Future<void> _onStateChanged(AppLifecycleState event) async {
    if (event == AppLifecycleState.hidden) {
      /// As the base case we enable the trigger when the app is put into the background and biometrics are enabled.
      /// We will disable the trigger when the app is unlocked, or brought back to the foreground when it's (still) unlocked
      /// The reason we enable the trigger eagerly is because doing so when the app is locked can be too late, e.g. when:
      /// 1. The user backgrounds the app
      /// 2. The "backgroundLockTimeout" time passes (i.e. the app will be locked on a resume callback)
      /// 3. The app is brought to the foreground, and the [AutoLockObserver] checks the timeout and locks the app
      /// 4. The [isLockedStream] fires while the app is in the resumed-state, meaning we can not distinguish (here) between
      ///     a manual, foreground or background (the case described here) lock event. Since the first two cases should
      ///     not cause [_shouldTriggerUnlock] to be set to true, we can't make a proper decision. We CAN detect the opposite,
      ///     as described in the 'if' statement below.
      final enabled = await _isBiometricLoginEnabledUseCase.invoke();
      _shouldTriggerUnlock = enabled;
    }
    if (event == AppLifecycleState.resumed && _shouldTriggerUnlock) {
      /// Set the [_shouldTriggerUnlock] flag to false when the app is brought to the foreground while it's unlocked.
      /// This makes sure we don't show the biometrics when:
      /// 1. Biometrics are enabled
      /// 2. The user backgrounds the app while it's unlocked
      /// 3. The user brings the app back to the foreground (before it automatically locks)
      /// 4. The user locks the app from the settings
      /// The debounce is used to make sure we get the latest locked state (i.e. avoid race condition of locking when
      /// the app is resumed).
      const debounceTime = Duration(milliseconds: 50);
      final locked = await _walletRepository.isLockedStream.debounceTime(debounceTime).first;
      if (!locked) _shouldTriggerUnlock = false;
    }
  }

  void _onLockChanged(bool locked) {
    /// When we unlock the app, we want to make sure it's not triggered again before the app is hidden.
    if (!locked) _shouldTriggerUnlock = false;
  }
}
