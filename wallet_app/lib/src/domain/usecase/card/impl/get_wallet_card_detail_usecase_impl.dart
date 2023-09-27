import 'package:fimber/fimber.dart';

import '../../../../../environment.dart';
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

    try {
      Organization organization = (await _organizationRepository.read(card.issuerId))!;
      InteractionTimelineAttribute? interaction = await _timelineAttributeRepository.readMostRecentInteraction(
        cardId,
        InteractionStatus.success,
      );
      OperationTimelineAttribute? operation = await _timelineAttributeRepository.readMostRecentOperation(
        cardId,
        OperationStatus.issued,
      );

      return WalletCardDetail(
        card: card,
        issuer: organization,
        latestSuccessInteraction: interaction,
        latestIssuedOperation: operation,
      );
    } catch (ex) {
      if (Environment.mockRepositories) rethrow;
      //FIXME: Fetch organization info through wallet_core
      Fimber.e('Could not load card details because we\'re running a core build. Enriching with mock data.', ex: ex);
      return WalletCardDetail(
        card: card,
        issuer: const Organization(id: '', name: 'RvIG', shortName: 'RvIG', category: '', description: '', logoUrl: ''),
        latestIssuedOperation: null,
        latestSuccessInteraction: null,
      );
    }
  }
}
