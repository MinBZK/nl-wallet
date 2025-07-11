import '../../model/card/wallet_card.dart';
import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class GetWalletCardUseCase extends WalletUseCase {
  Future<Result<WalletCard>> invoke(String attestationId);
}
