import '../../model/wallet_card.dart';

abstract class GetWalletCardUpdateIssuanceRequestIdUseCase {
  Future<String?> invoke(WalletCard card);
}
