import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../model/card/wallet_card.dart';
import '../../wallet_usecase.dart';
import '../observe_wallet_cards_usecase.dart';

class ObserveWalletCardsUseCaseImpl extends ObserveWalletCardsUseCase {
  final WalletCardRepository _walletCardRepository;

  ObserveWalletCardsUseCaseImpl(this._walletCardRepository);

  @override
  Stream<List<WalletCard>> invoke() =>
      _walletCardRepository.observeWalletCards().handleAppError('Observing wallet cards failed');
}
