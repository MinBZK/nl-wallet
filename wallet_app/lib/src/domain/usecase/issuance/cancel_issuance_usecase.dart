import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class CancelIssuanceUseCase extends WalletUseCase {
  Future<Result<void>> invoke();
}
