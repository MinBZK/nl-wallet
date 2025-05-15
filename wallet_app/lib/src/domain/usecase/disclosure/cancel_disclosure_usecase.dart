import '../../model/result/result.dart';
import '../wallet_usecase.dart';

export '../../../data/repository/disclosure/disclosure_repository.dart';

/// Cancels the ongoing disclosure session and returns the returnUrl to redirect the user (when available).
abstract class CancelDisclosureUseCase extends WalletUseCase {
  Future<Result<String?>> invoke();
}
