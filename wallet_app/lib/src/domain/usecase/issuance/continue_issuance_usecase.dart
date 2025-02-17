import '../../model/issuance/continue_issuance_result.dart';
import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class ContinueIssuanceUseCase extends WalletUseCase {
  Future<Result<ContinueIssuanceResult>> invoke();
}
