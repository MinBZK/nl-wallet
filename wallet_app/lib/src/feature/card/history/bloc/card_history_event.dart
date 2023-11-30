part of 'card_history_bloc.dart';

abstract class CardHistoryEvent extends Equatable {
  const CardHistoryEvent();
}

class CardHistoryLoadTriggered extends CardHistoryEvent {
  final String docType;

  const CardHistoryLoadTriggered(this.docType);

  @override
  List<Object?> get props => [docType];
}
