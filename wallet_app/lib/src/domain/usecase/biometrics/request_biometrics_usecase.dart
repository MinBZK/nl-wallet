import 'biometric_authentication_result.dart';

abstract class RequestBiometricsUsecase {
  Future<BiometricAuthenticationResult> invoke();
}
