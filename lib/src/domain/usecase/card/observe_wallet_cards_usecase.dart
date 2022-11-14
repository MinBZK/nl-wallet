import '../../../data/repository/card/wallet_card_repository.dart';
import '../../model/wallet_card.dart';

class ObserveWalletCardsUseCase {
  final WalletCardRepository walletCardRepository;

  ObserveWalletCardsUseCase(this.walletCardRepository);

  Stream<List<WalletCard>> invoke() {
    return walletCardRepository.observeWalletCards().map((cards) {
      return cards..sort((WalletCard a, WalletCard b) => a.id.compareTo(b.id));
    });
  }
}
