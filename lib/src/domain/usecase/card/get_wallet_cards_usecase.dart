import '../../../data/repository/card/wallet_card_repository.dart';
import '../../model/wallet_card.dart';

class GetWalletCardsUseCase {
  final WalletCardRepository walletCardRepository;

  GetWalletCardsUseCase(this.walletCardRepository);

  Future<List<WalletCard>> getWalletCardsOrderedByIdAsc() async {
    List<WalletCard> results = await walletCardRepository.getWalletCards();
    results.sort((WalletCard a, WalletCard b) => a.id.compareTo(b.id));
    return results;
  }
}
