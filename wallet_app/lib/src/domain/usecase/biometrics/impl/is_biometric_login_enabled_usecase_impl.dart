import '../../../../data/repository/biometric/biometric_repository.dart';
import '../../../../data/repository/wallet/wallet_repository.dart';
import '../is_biometric_login_enabled_usecase.dart';

class IsBiometricLoginEnabledUseCaseImpl extends IsBiometricLoginEnabledUseCase {
  final BiometricRepository _biometricRepository;
  final WalletRepository _walletRepository;

  IsBiometricLoginEnabledUseCaseImpl(
    this._biometricRepository,
    this._walletRepository,
  );

  @override
  Future<bool> invoke() async {
    /// Wallet needs to be registered before [isBiometricLoginEnabled] can be checked.
    if (await _walletRepository.isRegistered()) {
      return _biometricRepository.isBiometricLoginEnabled();
    }
    return false;
  }
}
