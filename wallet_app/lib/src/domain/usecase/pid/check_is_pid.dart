import '../../model/card/wallet_card.dart';
import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class CheckIsPidUseCase extends WalletUseCase {
  Future<Result<bool>> invoke(WalletCard card);
}
