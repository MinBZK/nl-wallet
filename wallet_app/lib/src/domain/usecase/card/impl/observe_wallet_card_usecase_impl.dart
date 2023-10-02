import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../model/wallet_card.dart';
import '../observe_wallet_card_usecase.dart';

class ObserveWalletCardUseCaseImpl implements ObserveWalletCardUseCase {
  final WalletCardRepository walletCardRepository;

  ObserveWalletCardUseCaseImpl(this.walletCardRepository);

  @override
  Stream<WalletCard> invoke(String cardId) {
    return walletCardRepository.observeWalletCards().map((cards) {
      return cards.firstWhere((card) => card.id == cardId);
    });
  }
}
