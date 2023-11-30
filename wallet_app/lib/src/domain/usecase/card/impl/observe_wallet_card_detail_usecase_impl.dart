import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../../data/repository/history/timeline_attribute_repository.dart';
import '../../../../data/repository/organization/organization_repository.dart';
import '../../../model/timeline/interaction_timeline_attribute.dart';
import '../../../model/timeline/operation_timeline_attribute.dart';
import '../../../model/wallet_card.dart';
import '../../../model/wallet_card_detail.dart';
import '../observe_wallet_card_detail_usecase.dart';

class ObserveWalletCardDetailUseCaseImpl implements ObserveWalletCardDetailUseCase {
  final WalletCardRepository _walletCardRepository;
  final OrganizationRepository _organizationRepository;
  final TimelineAttributeRepository _timelineAttributeRepository;

  ObserveWalletCardDetailUseCaseImpl(
    this._walletCardRepository,
    this._organizationRepository,
    this._timelineAttributeRepository,
  );

  @override
  Stream<WalletCardDetail> invoke(String cardId) {
    return _walletCardRepository
        .observeWalletCards()
        .map((cards) => cards.firstWhere((card) => card.id == cardId))
        .asyncMap((card) async => await _getWalletCardDetail(card));
  }

  Future<WalletCardDetail> _getWalletCardDetail(WalletCard card) async {
    Organization? organization = await _organizationRepository.findIssuer(card.docType);
    InteractionTimelineAttribute? interaction = await _timelineAttributeRepository.readMostRecentInteraction(
      card.docType,
      InteractionStatus.success,
    );
    OperationTimelineAttribute? operation = await _timelineAttributeRepository.readMostRecentOperation(
      card.docType,
      OperationStatus.issued,
    );
    return WalletCardDetail(
      card: card,
      issuer: organization!, // Exception handled upstream
      latestSuccessInteraction: interaction,
      latestIssuedOperation: operation,
    );
  }
}
