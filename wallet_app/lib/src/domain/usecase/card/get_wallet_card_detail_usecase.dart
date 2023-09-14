import '../../model/wallet_card_detail.dart';

abstract class GetWalletCardDetailUseCase {
  Future<WalletCardDetail> invoke(String cardId);
}
