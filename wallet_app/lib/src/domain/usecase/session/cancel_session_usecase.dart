import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class CancelSessionUseCase extends WalletUseCase {
  /// Cancels any active `disclosure` / `(pid) issuance / close proximity` session
  ///
  /// Returns a [returnUrl] if relevant (and available) for the cancelled session.
  Future<Result<String?>> invoke();
}
