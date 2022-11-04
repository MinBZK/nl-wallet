import '../../../domain/model/wallet_card.dart';

abstract class WalletCardRepository {
  Future<List<WalletCard>> getWalletCards();

  Future<WalletCard> getWalletCard(String cardId);
}
