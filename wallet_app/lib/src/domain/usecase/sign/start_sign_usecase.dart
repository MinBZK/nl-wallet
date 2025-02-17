import '../../model/result/result.dart';
import '../../model/start_sign_result/start_sign_result.dart';
import '../wallet_usecase.dart';

abstract class StartSignUseCase extends WalletUseCase {
  Future<Result<StartSignResult>> invoke(String signUri);
}
