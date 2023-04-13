part of 'card_data_bloc.dart';

abstract class CardDataEvent extends Equatable {
  const CardDataEvent();
}

class CardDataLoadTriggered extends CardDataEvent {
  final String cardId;

  const CardDataLoadTriggered(this.cardId);

  @override
  List<Object?> get props => [cardId];
}
