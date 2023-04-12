import '../../../domain/model/wallet_card.dart';

abstract class WalletCardRepository {
  Stream<List<WalletCard>> observeWalletCards();

  Future<bool> exists(String cardId);

  Future<void> create(WalletCard card);

  Future<List<WalletCard>> readAll();

  Future<WalletCard> read(String cardId);

  Future<void> update(WalletCard card);

  Future<void> delete(String cardId);
}
