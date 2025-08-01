import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../model/card/wallet_card.dart';
import '../../../model/result/result.dart';
import '../get_wallet_cards_usecase.dart';

class GetWalletCardsUseCaseImpl extends GetWalletCardsUseCase {
  final WalletCardRepository _walletCardRepository;

  GetWalletCardsUseCaseImpl(this._walletCardRepository);

  @override
  Future<Result<List<WalletCard>>> invoke() async {
    return tryCatch(
      () async => _walletCardRepository.readAll(),
      'Failed to get wallet cards',
    );
  }
}
