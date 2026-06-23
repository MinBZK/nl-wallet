import '../../model/card/wallet_card.dart';
import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class ContinueIssuanceUseCase extends WalletUseCase {
  Future<Result<List<WalletCard>>> invoke(String uri);
}
