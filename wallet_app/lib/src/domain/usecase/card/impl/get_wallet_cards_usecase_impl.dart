import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../model/card/wallet_card.dart';
import '../../../model/result/result.dart';
import '../get_wallet_cards_usecase.dart';

class GetWalletCardsUseCaseImpl extends GetWalletCardsUseCase {
  final WalletCardRepository walletCardRepository;

  GetWalletCardsUseCaseImpl(this.walletCardRepository);

  @override
  Future<Result<List<WalletCard>>> invoke() async {
    return tryCatch(
      () async => walletCardRepository.observeWalletCards().first.timeout(const Duration(seconds: 5)),
      'Failed to get wallet cards',
    );
  }
}
