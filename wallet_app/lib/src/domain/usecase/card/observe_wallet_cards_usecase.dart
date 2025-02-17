import '../../model/wallet_card.dart';
import '../wallet_usecase.dart';

abstract class ObserveWalletCardsUseCase extends WalletUseCase {
  Stream<List<WalletCard>> invoke();
}
