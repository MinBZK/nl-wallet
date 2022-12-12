import '../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../data/repository/card/wallet_card_repository.dart';
import '../../../feature/verification/model/organization.dart';
import '../../../wallet_constants.dart';
import '../../model/timeline_attribute.dart';
import '../../model/wallet_card.dart';

class WalletAddIssuedCardUseCase {
  final WalletCardRepository walletCardRepository;
  final TimelineAttributeRepository timelineAttributeRepository;

  WalletAddIssuedCardUseCase(this.walletCardRepository, this.timelineAttributeRepository);

  Future<void> invoke(WalletCard card, Organization organization) async {
    await Future.delayed(kDefaultMockDelay);

    final bool cardExistsInWallet = await walletCardRepository.exists(card.id);
    if (!cardExistsInWallet) {
      walletCardRepository.create(card);
      _createTimelineEntry(OperationType.issued, card, organization);
    } else {
      walletCardRepository.update(card);
      _createTimelineEntry(OperationType.renewed, card, organization);
    }
  }

  void _createTimelineEntry(OperationType operationType, WalletCard card, Organization organization) {
    timelineAttributeRepository.create(
      card.id,
      OperationAttribute(
        operationType: operationType,
        cardTitle: card.front.title,
        dateTime: DateTime.now(),
        organization: organization,
        attributes: card.attributes,
      ),
    );
    return;
  }
}
