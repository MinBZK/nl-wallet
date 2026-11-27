import '../../model/issuance/continue_pid_issuance_result.dart';
import '../../model/result/result.dart';
import '../wallet_usecase.dart';

export '../../model/issuance/continue_pid_issuance_result.dart';

abstract class ContinuePidIssuanceUseCase extends WalletUseCase {
  Future<Result<ContinuePidIssuanceResult>> invoke(String uri);
}
