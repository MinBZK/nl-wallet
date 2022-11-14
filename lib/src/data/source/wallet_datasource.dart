import '../../domain/model/wallet_card.dart';

abstract class WalletDataSource {
  Future<void> create(WalletCard card);

  Future<WalletCard?> read(String cardId);

  Future<void> update(WalletCard card);

  Future<void> delete(String cardId);

  Future<void> addInteraction(String cardId, String interaction);

  Future<List<String>> getInteractions(String cardId);

  Future<List<WalletCard>> readAll();

  Stream<List<WalletCard>> observeCards();

  Stream<List<String>> observeInteractions(String cardId);
}
