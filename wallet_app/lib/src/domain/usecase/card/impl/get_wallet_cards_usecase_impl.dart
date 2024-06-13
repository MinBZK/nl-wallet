import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../model/wallet_card.dart';
import '../get_wallet_cards_usecase.dart';

class GetWalletCardsUseCaseImpl implements GetWalletCardsUseCase {
  final WalletCardRepository walletCardRepository;

  GetWalletCardsUseCaseImpl(this.walletCardRepository);

  @override
  Future<List<WalletCard>> invoke() async {
    final List<WalletCard> results =
        await walletCardRepository.observeWalletCards().first.timeout(const Duration(seconds: 5));
    return List.from(results);
  }
}
