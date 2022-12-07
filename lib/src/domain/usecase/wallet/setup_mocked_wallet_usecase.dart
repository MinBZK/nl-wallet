import '../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../data/repository/card/wallet_card_repository.dart';
import '../../../data/repository/issuance/issuance_response_repository.dart';
import '../../../data/repository/wallet/wallet_repository.dart';
import '../../model/issuance_response.dart';
import '../../model/timeline_attribute.dart';

class SetupMockedWalletUseCase {
  final WalletRepository walletRepository;
  final WalletCardRepository walletCardRepository;
  final IssuanceResponseRepository issuanceResponseRepository;
  final TimelineAttributeRepository timelineAttributeRepository;

  SetupMockedWalletUseCase(
    this.walletRepository,
    this.walletCardRepository,
    this.issuanceResponseRepository,
    this.timelineAttributeRepository,
  );

  Future<void> invoke() async {
    // Create wallet
    await walletRepository.createWallet('000000');
    walletRepository.unlockWallet('000000');

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
  OperationAttribute(
    operationType: OperationType.issued,
    dateTime: DateTime.now().subtract(const Duration(days: 68)),
    cardTitle: 'Persoonsgegevens',
  ),
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
];
