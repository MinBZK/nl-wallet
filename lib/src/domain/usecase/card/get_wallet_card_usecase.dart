import '../../../data/repository/card/wallet_card_repository.dart';
import '../../model/wallet_card.dart';

class GetWalletCardUseCase {
  final WalletCardRepository walletCardRepository;

  GetWalletCardUseCase(this.walletCardRepository);

  Future<WalletCard> invoke(String cardId) async {
    WalletCard result = await walletCardRepository.read(cardId);
    return result;
  }
}
