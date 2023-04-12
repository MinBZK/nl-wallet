import '../../model/wallet_card.dart';

abstract class GetWalletCardsUseCase {
  Future<List<WalletCard>> invoke();
}
