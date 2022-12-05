import '../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../data/repository/card/wallet_card_repository.dart';
import '../../model/timeline_attribute.dart';
import '../../model/wallet_card.dart';

class CreateCardUseCase {
  final WalletCardRepository walletCardRepository;
  final TimelineAttributeRepository timelineAttributeRepository;

  CreateCardUseCase(this.walletCardRepository, this.timelineAttributeRepository);

  Future<void> invoke(WalletCard card) async {
    walletCardRepository.create(card);
    // Create the 'issued' history entry
    timelineAttributeRepository.create(
      card.id,
      OperationAttribute(
        operationType: OperationType.issued,
        cardTitle: card.front.title,
        dateTime: DateTime.now(),
      ),
    );
  }
}
