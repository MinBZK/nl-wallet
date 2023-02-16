import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../model/wallet_card.dart';
import '../observe_wallet_cards_usecase.dart';

class ObserveWalletCardsUseCaseImpl implements ObserveWalletCardsUseCase {
  final WalletCardRepository walletCardRepository;

  ObserveWalletCardsUseCaseImpl(this.walletCardRepository);

  @override
  Stream<List<WalletCard>> invoke() {
    return walletCardRepository.observeWalletCards();
  }
}
