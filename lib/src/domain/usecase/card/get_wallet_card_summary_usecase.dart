import '../../../data/repository/card/wallet_card_data_repository.dart';
import '../../../data/repository/card/wallet_card_repository.dart';
import '../../../data/repository/card/wallet_card_usage_repository.dart';
import '../../model/wallet_card.dart';
import '../../model/wallet_card_data_attribute.dart';
import '../../model/wallet_card_summary.dart';

class GetWalletCardSummaryUseCase {
  final WalletCardRepository walletCardRepository;
  final WalletCardDataRepository walletCardDataRepository;
  final WalletCardUsageRepository walletCardUsageRepository;

  GetWalletCardSummaryUseCase(
    this.walletCardRepository,
    this.walletCardDataRepository,
    this.walletCardUsageRepository,
  );

  Future<WalletCardSummary> getWalletCardSummary(String cardId) async {
    WalletCard card = await walletCardRepository.getWalletCard(cardId);
    WalletCardDataAttribute data = await walletCardDataRepository.getHighlight(cardId);
    WalletCardDataAttribute usage = await walletCardUsageRepository.getHighlight(cardId);

    WalletCardSummary summary = WalletCardSummary(
      card: card,
      data: data,
      usage: usage,
    );

    return summary;
  }
}
