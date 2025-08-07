part of 'card_detail_bloc.dart';

abstract class CardDetailEvent extends Equatable {
  const CardDetailEvent();
}

class CardDetailLoadTriggered extends CardDetailEvent {
  final String attestationId;

  const CardDetailLoadTriggered(this.attestationId);

  @override
  List<Object?> get props => [attestationId];
}
