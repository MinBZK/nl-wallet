import '../../model/result/result.dart';
import '../wallet_usecase.dart';

/// Update the flag to indicate the user has saved the revocation code.
abstract class SetRevocationCodeSavedUseCase extends WalletUseCase {
  Future<Result<void>> invoke({required bool saved});
}
