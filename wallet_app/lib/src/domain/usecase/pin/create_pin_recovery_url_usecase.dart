import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class CreatePinRecoveryRedirectUriUseCase extends WalletUseCase {
  Future<Result<String>> invoke();
}
