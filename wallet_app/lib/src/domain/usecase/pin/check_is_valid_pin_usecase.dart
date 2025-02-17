import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class CheckIsValidPinUseCase extends WalletUseCase {
  /// Validates the supplied [pin]
  ///
  /// Returns a [ValidatePinError] if the pin does
  /// not meet the required standards.
  Future<Result<void>> invoke(String pin);
}
