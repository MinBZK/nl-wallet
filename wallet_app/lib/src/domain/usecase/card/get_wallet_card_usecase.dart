import '../../model/result/result.dart';
import '../../model/wallet_card.dart';
import '../wallet_usecase.dart';

abstract class GetWalletCardUseCase extends WalletUseCase {
  Future<Result<WalletCard>> invoke(String docType);
}
