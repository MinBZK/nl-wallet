import '../../model/result/result.dart';
import '../wallet_usecase.dart';

/// Cancels the ongoing issuance session and returns the returnUrl to redirect the user (when available).
abstract class CancelIssuanceUseCase extends WalletUseCase {
  Future<Result<String?>> invoke();
}
