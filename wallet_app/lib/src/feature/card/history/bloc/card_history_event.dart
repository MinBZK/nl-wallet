part of 'card_history_bloc.dart';

abstract class CardHistoryEvent extends Equatable {
  const CardHistoryEvent();
}

class CardHistoryLoadTriggered extends CardHistoryEvent {
  final String cardId;

  const CardHistoryLoadTriggered(this.cardId);

  @override
  List<Object?> get props => [cardId];
}
