import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class AcceptIssuanceUseCase extends WalletUseCase {
  Future<Result<void>> invoke(Iterable<String> cardDocTypes);
}
