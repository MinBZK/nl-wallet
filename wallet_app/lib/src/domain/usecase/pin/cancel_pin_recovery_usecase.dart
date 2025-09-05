import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class CancelPinRecoveryUseCase extends WalletUseCase {
  Future<Result<void>> invoke();
}
