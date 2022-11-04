part of 'card_summary_bloc.dart';

abstract class CardSummaryEvent extends Equatable {
  const CardSummaryEvent();
}

class CardSummaryLoadTriggered extends CardSummaryEvent {
  final String id;

  const CardSummaryLoadTriggered(this.id);

  @override
  List<Object?> get props => [];
}
