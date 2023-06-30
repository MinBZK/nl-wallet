import '../../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../../feature/verification/model/organization.dart';
import '../../../model/timeline/operation_timeline_attribute.dart';
import '../../../model/wallet_card.dart';
import '../wallet_add_issued_cards_usecase.dart';

/// Adds [WalletCard](s) & [TimelineAttribute](s) to wallet
class WalletAddIssuedCardsUseCaseImpl extends WalletAddIssuedCardsUseCase {
  final WalletCardRepository walletCardRepository;
  final TimelineAttributeRepository timelineAttributeRepository;

  WalletAddIssuedCardsUseCaseImpl(this.walletCardRepository, this.timelineAttributeRepository);

  @override
  Future<void> invoke(List<WalletCard> cards, Organization organization) async {
    for (final card in cards) {
      final bool cardExistsInWallet = await walletCardRepository.exists(card.id);
      if (!cardExistsInWallet) {
        walletCardRepository.create(card);
        _createTimelineEntry(OperationStatus.issued, card, organization);
      } else {
        walletCardRepository.update(card);
        _createTimelineEntry(OperationStatus.renewed, card, organization);
      }
    }
  }

  void _createTimelineEntry(OperationStatus status, WalletCard card, Organization organization) {
    timelineAttributeRepository.create(
      OperationTimelineAttribute(
        status: status,
        cardTitle: card.front.title,
        dateTime: DateTime.now(),
        organization: organization,
        dataAttributes: card.attributes,
      ),
    );
    return;
  }
}
