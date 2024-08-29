import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../biometric_repository.dart';

class CoreBiometricRepository implements BiometricRepository {
  final TypedWalletCore _typedWalletCore;

  CoreBiometricRepository(this._typedWalletCore);

  @override
  Future<void> disableBiometricLogin() => _typedWalletCore.setBiometricUnlock(enabled: false);

  @override
  Future<void> enableBiometricLogin() => _typedWalletCore.setBiometricUnlock(enabled: true);

  @override
  Future<bool> isBiometricLoginEnabled() => _typedWalletCore.isBiometricLoginEnabled();
}
