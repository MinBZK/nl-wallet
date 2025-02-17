import '../../model/wallet_card.dart';
import '../wallet_usecase.dart';

abstract class ObserveWalletCardUseCase extends WalletUseCase {
  Stream<WalletCard> invoke(String cardId);
}
