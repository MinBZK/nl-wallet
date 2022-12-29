import '../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../data/repository/card/wallet_card_repository.dart';
import '../../../feature/verification/model/organization.dart';
import '../../model/timeline/timeline_attribute.dart';
import '../../model/wallet_card.dart';

class WalletAddIssuedCardUseCase {
  final WalletCardRepository walletCardRepository;
  final TimelineAttributeRepository timelineAttributeRepository;

  WalletAddIssuedCardUseCase(this.walletCardRepository, this.timelineAttributeRepository);

  Future<void> invoke(WalletCard card, Organization organization) async {
    final bool cardExistsInWallet = await walletCardRepository.exists(card.id);
    if (!cardExistsInWallet) {
      walletCardRepository.create(card);
      _createTimelineEntry(OperationStatus.issued, card, organization);
    } else {
      walletCardRepository.update(card);
      _createTimelineEntry(OperationStatus.renewed, card, organization);
    }
  }

  void _createTimelineEntry(OperationStatus status, WalletCard card, Organization organization) {
    timelineAttributeRepository.create(
      card.id,
      OperationAttribute(
        status: status,
        cardTitle: card.front.title,
        dateTime: DateTime.now(),
        organization: organization,
        attributes: card.attributes,
        isSession: false,
      ),
    );
    return;
  }
}
