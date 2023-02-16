import '../../model/wallet_card_summary.dart';

abstract class GetWalletCardSummaryUseCase {
  Future<WalletCardSummary> invoke(String cardId);
}
