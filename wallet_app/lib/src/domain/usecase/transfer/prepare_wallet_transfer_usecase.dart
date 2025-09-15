import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class PrepareWalletTransferUseCase extends WalletUseCase {
  Future<Result<String>> invoke();
}
