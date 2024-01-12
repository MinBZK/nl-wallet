import '../../model/wallet_card.dart';

abstract class GetWalletCardUseCase {
  Future<WalletCard> invoke(String docType);
}
