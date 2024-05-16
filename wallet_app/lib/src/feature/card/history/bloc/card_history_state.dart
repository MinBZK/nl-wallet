part of 'card_history_bloc.dart';

sealed class CardHistoryState extends Equatable {
  const CardHistoryState();
}

class CardHistoryInitial extends CardHistoryState {
  @override
  List<Object> get props => [];
}

class CardHistoryLoadInProgress extends CardHistoryState {
  const CardHistoryLoadInProgress();

  @override
  List<Object?> get props => [];
}

class CardHistoryLoadSuccess extends CardHistoryState {
  final WalletCard card;
  final List<WalletEvent> events;

  const CardHistoryLoadSuccess(this.card, this.events);

  @override
  List<Object> get props => [card, events];
}

class CardHistoryLoadFailure extends CardHistoryState {
  const CardHistoryLoadFailure();

  @override
  List<Object> get props => [];
}
