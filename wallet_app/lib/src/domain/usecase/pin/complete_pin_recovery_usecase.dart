import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class CompletePinRecoveryUseCase extends WalletUseCase {
  Future<Result<void>> invoke(String pin);
}
