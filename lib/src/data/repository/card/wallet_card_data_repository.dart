import '../../../domain/model/wallet_card_data_attribute.dart';

abstract class WalletCardDataRepository {
  Future<List<WalletCardDataAttribute>> getAll(String cardId);

  Future<WalletCardDataAttribute> getHighlight(String cardId);
}
