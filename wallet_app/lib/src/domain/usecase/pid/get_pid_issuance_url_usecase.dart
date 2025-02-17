import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class GetPidIssuanceUrlUseCase extends WalletUseCase {
  Future<Result<String>> invoke();
}
