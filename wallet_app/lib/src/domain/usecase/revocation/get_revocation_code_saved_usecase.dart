import '../../model/result/result.dart';
import '../wallet_usecase.dart';

/// Read the flag that indicates the user has saved the revocation code.
abstract class GetRevocationCodeSavedUseCase extends WalletUseCase {
  Future<Result<bool>> invoke();
}
