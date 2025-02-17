import '../../model/issuance/start_issuance_result.dart';
import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class StartIssuanceUseCase extends WalletUseCase {
  Future<Result<StartIssuanceResult>> invoke(String issuanceUri);
}
