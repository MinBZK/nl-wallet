import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class StartCloseProximityDisclosureUseCase extends WalletUseCase {
  Future<Result<String>> invoke();
}
