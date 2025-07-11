import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../model/card/wallet_card.dart';
import '../../wallet_usecase.dart';
import '../observe_wallet_card_usecase.dart';

class ObserveWalletCardUseCaseImpl extends ObserveWalletCardUseCase {
  final WalletCardRepository walletCardRepository;

  ObserveWalletCardUseCaseImpl(this.walletCardRepository);

  @override
  Stream<WalletCard> invoke(String cardId) {
    return walletCardRepository.observeWalletCards().map((cards) {
      return cards.firstWhere((card) => card.attestationId == cardId);
    }).handleAppError('Error while observing card with id $cardId');
  }
}
