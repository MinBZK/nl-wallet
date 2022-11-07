import '../../../data/repository/card/wallet_card_data_highlight_repository.dart';
import '../../../data/repository/card/wallet_card_repository.dart';
import '../../../data/repository/card/wallet_card_usage_attribute_repository.dart';
import '../../model/data_highlight.dart';
import '../../model/usage_attribute.dart';
import '../../model/wallet_card.dart';
import '../../model/wallet_card_summary.dart';

class GetWalletCardSummaryUseCase {
  final WalletCardRepository walletCardRepository;
  final WalletCardDataHighlightRepository walletCardDataHighlightRepository;
  final WalletCardUsageAttributeRepository walletCardUsageAttributeRepository;

  GetWalletCardSummaryUseCase(
    this.walletCardRepository,
    this.walletCardDataHighlightRepository,
    this.walletCardUsageAttributeRepository,
  );

  Future<WalletCardSummary> getWalletCardSummary(String cardId) async {
    WalletCard card = await walletCardRepository.getWalletCard(cardId);
    DataHighlight dataHighlight = await walletCardDataHighlightRepository.getLatest(cardId);
    UsageAttribute usageAttribute = await walletCardUsageAttributeRepository.getLatest(cardId);

    WalletCardSummary summary = WalletCardSummary(
      card: card,
      dataHighlight: dataHighlight,
      usageAttribute: usageAttribute,
    );

    return summary;
  }
}
