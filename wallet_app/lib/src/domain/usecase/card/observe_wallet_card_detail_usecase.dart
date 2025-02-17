import '../../model/wallet_card_detail.dart';
import '../wallet_usecase.dart';

abstract class ObserveWalletCardDetailUseCase extends WalletUseCase {
  Stream<WalletCardDetail> invoke(String cardId);
}
