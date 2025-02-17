import '../wallet_usecase.dart';

/// Checks if biometric unlock is enabled
abstract class IsBiometricLoginEnabledUseCase extends WalletUseCase {
  Future<bool> invoke();
}
