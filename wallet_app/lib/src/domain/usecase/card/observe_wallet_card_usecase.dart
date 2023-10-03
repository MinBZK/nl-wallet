import '../../model/wallet_card.dart';

abstract class ObserveWalletCardUseCase {
  Stream<WalletCard> invoke(String cardId);
}
