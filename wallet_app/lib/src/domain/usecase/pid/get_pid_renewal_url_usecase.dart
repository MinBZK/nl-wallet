import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class GetPidRenewalUrlUseCase extends WalletUseCase {
  Future<Result<String>> invoke();
}
