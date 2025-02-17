import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class ChangePinUseCase extends WalletUseCase {
  Future<Result<void>> invoke(String oldPin, String newPin);
}
