import '../../model/result/result.dart';
import '../wallet_usecase.dart';

/// Enable/Disable the usage of biometrics for
/// unlocking the app. Signing transactions will
/// still require the user's PIN.
///
/// [authenticateBeforeEnabling] dictates whether a successful biometrics authentication
/// should be performed before toggling the setting.
abstract class SetBiometricsUseCase extends WalletUseCase {
  Future<Result<void>> invoke({required bool enable, required bool authenticateBeforeEnabling});
}
