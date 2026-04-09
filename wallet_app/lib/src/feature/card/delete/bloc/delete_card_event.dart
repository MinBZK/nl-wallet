part of 'delete_card_bloc.dart';

abstract class DeleteCardEvent extends Equatable {
  const DeleteCardEvent();

  @override
  List<Object> get props => [];
}

class DeleteCardLoadTriggered extends DeleteCardEvent {
  final String attestationId;
  final String cardTitle;

  const DeleteCardLoadTriggered({required this.attestationId, required this.cardTitle});

  @override
  List<Object> get props => [attestationId, cardTitle];
}

class DeleteCardPinConfirmed extends DeleteCardEvent {
  const DeleteCardPinConfirmed();
}
