import '../../domain/model/wallet_card.dart';

abstract class WalletDataSource {
  Future<List<WalletCard>> readAll();

  Future<WalletCard?> read(String docType);

  Stream<List<WalletCard>> observeCards();
}
