import '../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../data/repository/card/wallet_card_repository.dart';
import '../../../data/repository/issuance/issuance_response_repository.dart';
import '../../model/issuance_response.dart';
import '../../model/timeline_attribute.dart';

/// Temporary usecase to make sure the app initializes with cards
class InitCardsUseCase {
  final WalletCardRepository walletCardRepository;
  final IssuanceResponseRepository issuanceResponseRepository;
  final TimelineAttributeRepository timelineAttributeRepository;

  InitCardsUseCase(
    this.walletCardRepository,
    this.issuanceResponseRepository,
    this.timelineAttributeRepository,
  ) {
    invoke();
  }

  void invoke() async {
    // Add PID card
    const cardId = 'PID_1';
    final IssuanceResponse issuanceResponse = await issuanceResponseRepository.read(cardId);
    walletCardRepository.create(issuanceResponse.cards.first);

    // Add PID card; history
    for (TimelineAttribute attribute in _kMockPidTimelineAttributes) {
      timelineAttributeRepository.create(cardId, attribute);
    }
  }
}

final List<TimelineAttribute> _kMockPidTimelineAttributes = [
  InteractionAttribute(
    interactionType: InteractionType.success,
    organization: 'Amsterdam Airport Schiphol',
    dateTime: DateTime.now().subtract(const Duration(hours: 4)),
  ),
  InteractionAttribute(
    interactionType: InteractionType.rejected,
    organization: 'Marktplaats',
    dateTime: DateTime.now().subtract(const Duration(days: 5)),
  ),
  InteractionAttribute(
    interactionType: InteractionType.rejected,
    organization: 'DUO',
    dateTime: DateTime.now().subtract(const Duration(days: 5)),
  ),
  OperationAttribute(
    operationType: OperationType.issued,
    description: 'Deze kaart is geldig tot november 2025',
    dateTime: DateTime.now().subtract(const Duration(days: 35)),
  ),
];
