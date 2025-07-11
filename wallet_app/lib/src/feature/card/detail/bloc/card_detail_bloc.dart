import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:rxdart/rxdart.dart';

import '../../../../domain/model/card/wallet_card.dart';
import '../../../../domain/model/wallet_card_detail.dart';
import '../../../../domain/usecase/card/observe_wallet_card_detail_usecase.dart';

part 'card_detail_event.dart';
part 'card_detail_state.dart';

/// This is more of a sanity check for programming errors, if notifyEntryTransitionCompleted is not called, this makes
/// sure we don't block on the completer forever.
const _kMaxEntryTransitionDuration = Duration(seconds: 5);

class CardDetailBloc extends Bloc<CardDetailEvent, CardDetailState> {
  final ObserveWalletCardDetailUseCase observeWalletCardDetailUseCase;
  final Completer<bool> _entryTransitionCompleted = Completer();

  CardDetailBloc(this.observeWalletCardDetailUseCase, WalletCard? preloadedCard)
      : super(preloadedCard == null ? CardDetailInitial() : CardDetailLoadInProgress(card: preloadedCard)) {
    on<CardDetailLoadTriggered>(_onCardDetailLoadTriggered);
  }

  Future<void> _onCardDetailLoadTriggered(CardDetailLoadTriggered event, emit) async {
    final state = this.state;
    bool loadTriggeredForVisibleCard = false;
    switch (state) {
      case CardDetailLoadInProgress():
        loadTriggeredForVisibleCard = state.card?.attestationId == event.cardId;
      case CardDetailLoadSuccess():
        loadTriggeredForVisibleCard = state.detail.card.attestationId == event.cardId;
      default:
    }
    if (!loadTriggeredForVisibleCard) {
      notifyEntryTransitionCompleted(); // We can't showing any [WalletCard] preview, make sure we never block on it.
      emit(const CardDetailLoadInProgress());
    }

    await emit.forEach(
      observeWalletCardDetailUseCase
          .invoke(event.cardId)
          .debounce((_) => Stream.fromFuture(_entryTransitionCompleted.future.timeout(_kMaxEntryTransitionDuration))),
      // ignore: unnecessary_lambdas, not actually unnecessary due to expected signature
      onData: (data) => CardDetailLoadSuccess(data),
      onError: (ex, stack) {
        //Note: when providing onError like this the subscription is not cancelled on errors
        Fimber.e('Failed to observe card detail', ex: ex, stacktrace: stack);
        return CardDetailLoadFailure(event.cardId);
      },
    );
  }

  void notifyEntryTransitionCompleted() {
    if (!_entryTransitionCompleted.isCompleted) _entryTransitionCompleted.complete(true);
  }
}
