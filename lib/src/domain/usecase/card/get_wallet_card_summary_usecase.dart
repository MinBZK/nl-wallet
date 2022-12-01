import '../../../data/repository/card/data_highlight_repository.dart';
import '../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../data/repository/card/wallet_card_repository.dart';
import '../../model/data_highlight.dart';
import '../../model/timeline_attribute.dart';
import '../../model/wallet_card.dart';
import '../../model/wallet_card_summary.dart';

class GetWalletCardSummaryUseCase {
  final WalletCardRepository walletCardRepository;
  final DataHighlightRepository walletCardDataHighlightRepository;
  final TimelineAttributeRepository timelineAttributeRepository;

  GetWalletCardSummaryUseCase(
    this.walletCardRepository,
    this.walletCardDataHighlightRepository,
    this.timelineAttributeRepository,
  );

  Future<WalletCardSummary> getSummary(String cardId) async {
    WalletCard card = await walletCardRepository.read(cardId);
    DataHighlight dataHighlight = await walletCardDataHighlightRepository.getLatest(cardId);
    InteractionAttribute? interactionAttribute = await timelineAttributeRepository.readLastInteraction(
      cardId,
      InteractionType.success,
    );

    WalletCardSummary summary = WalletCardSummary(
      card: card,
      dataHighlight: dataHighlight,
      interactionAttribute: interactionAttribute,
    );

    return summary;
  }
}
