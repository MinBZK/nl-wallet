part of 'card_history_bloc.dart';

abstract class CardHistoryEvent extends Equatable {
  const CardHistoryEvent();
}

class CardHistoryLoadTriggered extends CardHistoryEvent {
  final String attestationId;

  const CardHistoryLoadTriggered(this.attestationId);

  @override
  List<Object?> get props => [attestationId];
}
