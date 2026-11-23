import 'package:collection/collection.dart';
import 'package:rxdart/rxdart.dart';

import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../../data/repository/event/wallet_event_repository.dart';
import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../model/card/wallet_card.dart';
import '../../../model/event/wallet_event.dart';
import '../../../model/result/application_error.dart';
import '../../../model/wallet_card_detail.dart';
import '../../wallet_usecase.dart';
import '../observe_wallet_card_detail_usecase.dart';

class ObserveWalletCardDetailUseCaseImpl extends ObserveWalletCardDetailUseCase {
  final WalletCardRepository _walletCardRepository;
  final WalletEventRepository _walletEventRepository;
  final WalletRepository _walletRepository;

  ObserveWalletCardDetailUseCaseImpl(
    this._walletCardRepository,
    this._walletEventRepository,
    this._walletRepository,
  );

  @override
  Stream<WalletCardDetail> invoke(String cardId) {
    // Card updates also arrive while the wallet is locked (e.g. from the background revocation check), but core
    // rejects history reads then with a state error, which ends up on the invariant error screen. Re-subscribing
    // on unlock replays the latest cards, so the details are refreshed right away.
    return _walletRepository.isLockedStream
        .distinct()
        .switchMap((locked) => locked ? const Stream<WalletCardDetail>.empty() : _observeWalletCardDetail(cardId))
        .handleAppError('Error while observing card details');
  }

  Stream<WalletCardDetail> _observeWalletCardDetail(String cardId) {
    return _walletCardRepository
        .observeWalletCards()
        .map((cards) => cards.firstWhereOrNull((card) => card.attestationId == cardId))
        .whereNotNull()
        .asyncMap(_getWalletCardDetail);
  }

  Future<WalletCardDetail> _getWalletCardDetail(WalletCard card) async {
    if (!card.isPersisted) {
      throw GenericError(
        'Unknown card. cardId: ${card.attestationId}',
        sourceError: UnsupportedError('Details can only be fetched for persisted cards'),
      );
    }
    final DisclosureEvent? disclosureEvent = await _walletEventRepository.readMostRecentDisclosureEvent(
      card.attestationId!,
      EventStatus.success,
    );
    final IssuanceEvent? issuanceEvent = await _walletEventRepository.readMostRecentIssuanceEvent(
      card.attestationId!,
      EventStatus.success,
    );
    return WalletCardDetail(
      card: card,
      mostRecentSuccessfulDisclosure: disclosureEvent,
      mostRecentIssuance: issuanceEvent,
    );
  }
}
