part of 'card_detail_bloc.dart';

abstract class CardDetailEvent extends Equatable {
  const CardDetailEvent();
}

class CardDetailLoadTriggered extends CardDetailEvent {
  final String cardId;

  const CardDetailLoadTriggered(this.cardId);

  @override
  List<Object?> get props => [cardId];
}
