import 'package:collection/collection.dart';
import 'package:rxdart/transformers.dart';

import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../model/card/wallet_card.dart';
import '../../wallet_usecase.dart';
import '../observe_wallet_card_usecase.dart';

class ObserveWalletCardUseCaseImpl extends ObserveWalletCardUseCase {
  final WalletCardRepository _walletCardRepository;

  ObserveWalletCardUseCaseImpl(this._walletCardRepository);

  @override
  Stream<WalletCard> invoke(String cardId) {
    return _walletCardRepository
        .observeWalletCards()
        .map((cards) => cards.firstWhereOrNull((card) => card.attestationId == cardId))
        .whereNotNull()
        .handleAppError('Error while observing card with id $cardId');
  }
}
