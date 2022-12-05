import '../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../data/repository/card/wallet_card_repository.dart';
import '../../../wallet_constants.dart';
import '../../model/timeline_attribute.dart';
import '../../model/wallet_card.dart';

class WalletAddIssuedCardUseCase {
  final WalletCardRepository walletCardRepository;
  final TimelineAttributeRepository timelineAttributeRepository;

  WalletAddIssuedCardUseCase(this.walletCardRepository, this.timelineAttributeRepository);

  Future<void> invoke(WalletCard card) async {
    await Future.delayed(kDefaultMockDelay);

    final bool cardExistsInWallet = await walletCardRepository.exists(card.id);
    if (!cardExistsInWallet) {
      walletCardRepository.create(card);
      _createTimelineEntry(card, OperationType.issued);
    } else {
      walletCardRepository.update(card);
      _createTimelineEntry(card, OperationType.renewed);
    }
  }

  void _createTimelineEntry(WalletCard card, OperationType operationType) {
    timelineAttributeRepository.create(
      card.id,
      OperationAttribute(
        operationType: operationType,
        cardTitle: card.front.title,
        dateTime: DateTime.now(),
      ),
    );
    return;
  }
}
