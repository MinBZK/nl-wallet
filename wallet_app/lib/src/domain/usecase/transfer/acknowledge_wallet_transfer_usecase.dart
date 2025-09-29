import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class AcknowledgeWalletTransferUseCase extends WalletUseCase {
  Future<Result<void>> invoke(String uri);
}
