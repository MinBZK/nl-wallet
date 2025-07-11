import '../../../domain/model/card/wallet_card.dart';

abstract class WalletCardRepository {
  Stream<List<WalletCard>> observeWalletCards();

  Future<bool> exists(String attestationId);

  Future<List<WalletCard>> readAll();

  Future<WalletCard> read(String attestationId);
}
