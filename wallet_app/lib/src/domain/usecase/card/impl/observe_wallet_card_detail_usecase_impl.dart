import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../../data/repository/event/wallet_event_repository.dart';
import '../../../model/event/wallet_event.dart';
import '../../../model/wallet_card.dart';
import '../../../model/wallet_card_detail.dart';
import '../../wallet_usecase.dart';
import '../observe_wallet_card_detail_usecase.dart';

class ObserveWalletCardDetailUseCaseImpl extends ObserveWalletCardDetailUseCase {
  final WalletCardRepository _walletCardRepository;
  final WalletEventRepository _walletEventRepository;

  ObserveWalletCardDetailUseCaseImpl(
    this._walletCardRepository,
    this._walletEventRepository,
  );

  @override
  Stream<WalletCardDetail> invoke(String cardId) {
    return _walletCardRepository
        .observeWalletCards()
        .map((cards) => cards.firstWhere((card) => card.id == cardId))
        .asyncMap(_getWalletCardDetail)
        .handleAppError('Error while observing card details');
  }

  Future<WalletCardDetail> _getWalletCardDetail(WalletCard card) async {
    final DisclosureEvent? disclosureEvent = await _walletEventRepository.readMostRecentDisclosureEvent(
      card.docType,
      EventStatus.success,
    );
    final IssuanceEvent? issuanceEvent = await _walletEventRepository.readMostRecentIssuanceEvent(
      card.docType,
      EventStatus.success,
    );
    return WalletCardDetail(
      card: card,
      mostRecentSuccessfulDisclosure: disclosureEvent,
      mostRecentIssuance: issuanceEvent,
    );
  }
}
