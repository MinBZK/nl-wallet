import '../../model/result/result.dart';
import '../../model/wallet_card.dart';
import '../wallet_usecase.dart';

abstract class GetWalletCardsUseCase extends WalletUseCase {
  Future<Result<List<WalletCard>>> invoke();
}
