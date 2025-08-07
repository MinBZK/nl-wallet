import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet_mock/mock.dart';

import '../../../../domain/model/card/wallet_card.dart';
import '../../../../domain/model/wallet_card_detail.dart';
import '../../../../domain/usecase/card/observe_wallet_card_detail_usecase.dart';

part 'card_detail_event.dart';
part 'card_detail_state.dart';

/// This is more of a sanity check for programming errors, if notifyEntryTransitionCompleted is not called, this makes
/// sure we don't block on the completer forever.
const _kMaxEntryTransitionDuration = Duration(seconds: 5);

class CardDetailBloc extends Bloc<CardDetailEvent, CardDetailState> {
  final ObserveWalletCardDetailUseCase _observeWalletCardDetailUseCase;
  final Completer<bool> _entryTransitionCompleted = Completer();

  /// Fetch the attestationId associated to the bloc's current state.
  String? get attestationId {
    final state = this.state;
    switch (state) {
      case CardDetailInitial():
        return null;
      case CardDetailLoadInProgress():
        return state.card?.attestationId;
      case CardDetailLoadSuccess():
        return state.detail.card.attestationId;
      case CardDetailLoadFailure():
        return state.attestationId;
    }
  }

  CardDetailBloc(this._observeWalletCardDetailUseCase, WalletCard? preloadedCard)
      : super(preloadedCard == null ? CardDetailInitial() : CardDetailLoadInProgress(card: preloadedCard)) {
    on<CardDetailLoadTriggered>(_onCardDetailLoadTriggered);
  }

  Future<void> _onCardDetailLoadTriggered(CardDetailLoadTriggered event, emit) async {
    final bool loadTriggeredForVisibleCard = attestationId == event.attestationId;
    if (!loadTriggeredForVisibleCard) {
      notifyEntryTransitionCompleted(); // We can't show any [WalletCard] preview, make sure we never block on it.
      emit(const CardDetailLoadInProgress());
    }

    await emit.forEach(
      _observeWalletCardDetailUseCase
          .invoke(event.attestationId)
          .debounce((_) => Stream.fromFuture(_entryTransitionCompleted.future.timeout(_kMaxEntryTransitionDuration))),
      // ignore: unnecessary_lambdas, not actually unnecessary due to expected signature
      onData: (data) => CardDetailLoadSuccess(data, showRenewOption: _isCardRenewable(data)),
      onError: (ex, stack) {
        //Note: when providing onError like this the subscription is not cancelled on errors
        Fimber.e('Failed to observe card detail', ex: ex, stacktrace: stack);
        return CardDetailLoadFailure(event.attestationId);
      },
    );
  }

  // FIXME: This logic is to be implemented in the future, see PVW-4586 / PVW-4619
  bool _isCardRenewable(WalletCardDetail data) =>
      [MockAttestationTypes.pid, MockAttestationTypes.address].contains(data.card.attestationType);

  void notifyEntryTransitionCompleted() {
    if (!_entryTransitionCompleted.isCompleted) _entryTransitionCompleted.complete(true);
  }
}
