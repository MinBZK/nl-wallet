import '../../../domain/model/wallet_card_data_attribute.dart';

abstract class WalletCardUsageRepository {
  Future<WalletCardDataAttribute> getAll(String cardId);

  Future<WalletCardDataAttribute> getHighlight(String cardId);
}
