import '../../../data/repository/card/wallet_card_repository.dart';
import '../../model/wallet_card.dart';

class GetWalletCardsUseCase {
  final WalletCardRepository walletCardRepository;

  GetWalletCardsUseCase(this.walletCardRepository);

  Future<List<WalletCard>> invoke() async {
    List<WalletCard> results = await walletCardRepository.readAll();
    return List.from(results);
  }
}
