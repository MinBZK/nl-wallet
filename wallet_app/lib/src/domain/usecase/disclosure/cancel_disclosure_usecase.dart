import '../../model/result/result.dart';
import '../wallet_usecase.dart';

export '../../../data/repository/disclosure/disclosure_repository.dart';

abstract class CancelDisclosureUseCase extends WalletUseCase {
  /// Cancels the ongoing disclosure session and returns the returnUrl to redirect the user (when available).
  Future<Result<String?>> invoke();
}
