import '../../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../../data/repository/organization/organization_repository.dart';
import '../../../../feature/verification/model/organization.dart';
import '../../../model/timeline/interaction_timeline_attribute.dart';
import '../../../model/timeline/operation_timeline_attribute.dart';
import '../../../model/wallet_card.dart';
import '../../../model/wallet_card_detail.dart';
import '../get_wallet_card_detail_usecase.dart';

class GetWalletCardDetailUseCaseImpl implements GetWalletCardDetailUseCase {
  final WalletCardRepository _walletCardRepository;
  final OrganizationRepository _organizationRepository;
  final TimelineAttributeRepository _timelineAttributeRepository;

  GetWalletCardDetailUseCaseImpl(
    this._walletCardRepository,
    this._organizationRepository,
    this._timelineAttributeRepository,
  );

  @override
  Future<WalletCardDetail> invoke(String cardId) async {
    WalletCard card = await _walletCardRepository.read(cardId);
    Organization organization = (await _organizationRepository.read(card.issuerId))!;
    InteractionTimelineAttribute? interaction = await _timelineAttributeRepository.readMostRecentInteraction(
      cardId,
      InteractionStatus.success,
    );
    OperationTimelineAttribute? operation = await _timelineAttributeRepository.readMostRecentOperation(
      cardId,
      OperationStatus.issued,
    );

    WalletCardDetail detail = WalletCardDetail(
      card: card,
      issuer: organization,
      latestSuccessInteraction: interaction,
      latestIssuedOperation: operation,
    );

    return detail;
  }
}
