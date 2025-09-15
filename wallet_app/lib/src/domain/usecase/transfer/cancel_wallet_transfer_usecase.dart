import '../pin/check_pin_usecase.dart';
import '../wallet_usecase.dart';

abstract class CancelWalletTransferUseCase extends WalletUseCase {
  Future<Result<void>> invoke();
}
