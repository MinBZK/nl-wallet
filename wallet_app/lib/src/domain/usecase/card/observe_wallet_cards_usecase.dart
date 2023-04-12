import '../../model/wallet_card.dart';

abstract class ObserveWalletCardsUseCase {
  Stream<List<WalletCard>> invoke();
}
