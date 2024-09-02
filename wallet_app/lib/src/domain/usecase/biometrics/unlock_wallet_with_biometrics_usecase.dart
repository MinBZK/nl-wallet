import 'biometric_authentication_result.dart';

abstract class UnlockWalletWithBiometricsUseCase {
  Future<BiometricAuthenticationResult> invoke();
}
