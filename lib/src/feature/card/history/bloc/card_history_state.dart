part of 'card_history_bloc.dart';

abstract class CardHistoryState extends Equatable {
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
  final List<TimelineAttribute> attributes;

  const CardHistoryLoadSuccess(this.card, this.attributes);

  @override
  List<Object> get props => [card, attributes];
}

class CardHistoryLoadFailure extends CardHistoryState {
  const CardHistoryLoadFailure();

  @override
  List<Object> get props => [];
}
