import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class RejectSignAgreementUseCase extends WalletUseCase {
  Future<Result<void>> invoke();
}
