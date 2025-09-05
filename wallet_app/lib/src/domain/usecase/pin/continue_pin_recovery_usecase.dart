import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class ContinuePinRecoveryUseCase extends WalletUseCase {
  Future<Result<void>> invoke(String uri);
}
