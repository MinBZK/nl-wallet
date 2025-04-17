import '../../model/result/result.dart';
import '../wallet_usecase.dart';
import 'biometric_authentication_result.dart';

abstract class RequestBiometricsUseCase extends WalletUseCase {
  Future<Result<BiometricAuthenticationResult>> invoke();
}
