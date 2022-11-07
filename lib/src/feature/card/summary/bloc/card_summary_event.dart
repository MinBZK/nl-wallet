part of 'card_summary_bloc.dart';

abstract class CardSummaryEvent extends Equatable {
  const CardSummaryEvent();
}

class CardSummaryLoadTriggered extends CardSummaryEvent {
  final String cardId;

  const CardSummaryLoadTriggered(this.cardId);

  @override
  List<Object?> get props => [cardId];
}
