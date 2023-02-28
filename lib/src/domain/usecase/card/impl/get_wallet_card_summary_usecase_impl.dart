import '../../../../data/repository/card/data_highlight_repository.dart';
import '../../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../model/data_highlight.dart';
import '../../../model/timeline/interaction_timeline_attribute.dart';
import '../../../model/wallet_card.dart';
import '../../../model/wallet_card_summary.dart';
import '../get_wallet_card_summary_usecase.dart';

class GetWalletCardSummaryUseCaseImpl implements GetWalletCardSummaryUseCase {
  final WalletCardRepository walletCardRepository;
  final DataHighlightRepository walletCardDataHighlightRepository;
  final TimelineAttributeRepository timelineAttributeRepository;

  GetWalletCardSummaryUseCaseImpl(
    this.walletCardRepository,
    this.walletCardDataHighlightRepository,
    this.timelineAttributeRepository,
  );

  @override
  Future<WalletCardSummary> invoke(String cardId) async {
    WalletCard card = await walletCardRepository.read(cardId);
    DataHighlight dataHighlight = await walletCardDataHighlightRepository.getLatest(cardId);
    InteractionTimelineAttribute? interactionAttribute = await timelineAttributeRepository.readLastInteraction(
      cardId,
      InteractionStatus.success,
    );

    WalletCardSummary summary = WalletCardSummary(
      card: card,
      dataHighlight: dataHighlight,
      interactionAttribute: interactionAttribute,
    );

    return summary;
  }
}
