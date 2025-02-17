import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class CancelPidIssuanceUseCase extends WalletUseCase {
  /// Cancel the active pid issuance session. The return value indicates
  ///  if there was an ongoing session (that has now been cancelled).
  Future<Result<bool>> invoke();
}
