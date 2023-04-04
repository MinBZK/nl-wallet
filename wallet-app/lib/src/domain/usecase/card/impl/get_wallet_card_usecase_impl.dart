import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../model/wallet_card.dart';
import '../get_wallet_card_usecase.dart';

class GetWalletCardUseCaseImpl implements GetWalletCardUseCase {
  final WalletCardRepository walletCardRepository;

  GetWalletCardUseCaseImpl(this.walletCardRepository);

  @override
  Future<WalletCard> invoke(String cardId) async {
    WalletCard result = await walletCardRepository.read(cardId);
    return result;
  }
}
