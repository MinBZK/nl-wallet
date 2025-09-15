import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class InitWalletTransferUseCase extends WalletUseCase {
  Future<Result<void>> invoke();
}
