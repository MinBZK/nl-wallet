import 'dart:ui';

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
      /// Only enable the flag if biometrics are enabled
      final enabled = await _isBiometricLoginEnabledUseCase.invoke();
      _shouldTriggerUnlock = enabled;
    }
    if (event == AppLifecycleState.resumed) {
      /// Make sure the flag is set to false when the user hides and shows the
      /// app. This makes sure we don't show the biometrics when:
      /// The user hides the app, returns to the app, opens the menu & locks the app.
      final locked = await _walletRepository.isLockedStream.first;
      if (!locked) _shouldTriggerUnlock = false;
    }
  }

  void _onLockChanged(bool locked) {
    /// When we unlock the app, we want to make sure it's not triggered again before the app is hidden
    /// Mainly used as a backup, as we currently rely on [getAndSetShouldTriggerUnlock] to make sure its
    /// set to false after triggering the biometric unlock.
    if (!locked) _shouldTriggerUnlock = false;
  }
}
