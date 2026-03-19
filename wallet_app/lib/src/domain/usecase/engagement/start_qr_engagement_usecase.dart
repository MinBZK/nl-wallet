import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class StartQrEngagementUseCase extends WalletUseCase {
  Future<Result<String>> invoke();
}
