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
    await walletCardRepository.create(card);
    await timelineAttributeRepository.create(
      card.id,
      OperationAttribute(
        operationType: OperationType.issued,
        dateTime: DateTime.now(),
        description: '',
      ),
    );
    return;
  }
}
