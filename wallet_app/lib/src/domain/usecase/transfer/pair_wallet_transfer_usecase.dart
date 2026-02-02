import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class PairWalletTransferUseCase extends WalletUseCase {
  Future<Result<void>> invoke(String uri);
}
