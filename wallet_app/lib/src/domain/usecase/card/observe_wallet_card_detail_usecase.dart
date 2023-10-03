import '../../model/wallet_card_detail.dart';

abstract class ObserveWalletCardDetailUseCase {
  Stream<WalletCardDetail> invoke(String cardId);
}
