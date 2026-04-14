part of 'delete_card_bloc.dart';

sealed class DeleteCardState extends Equatable {
  const DeleteCardState();

  @override
  List<Object?> get props => [];
}

class DeleteCardInitial extends DeleteCardState {
  const DeleteCardInitial();
}

class DeleteCardProvidePin extends DeleteCardState {
  final String attestationId;
  final String cardTitle;

  const DeleteCardProvidePin({required this.attestationId, required this.cardTitle});

  @override
  List<Object?> get props => [attestationId, cardTitle];
}

class DeleteCardSuccess extends DeleteCardState {
  final String cardTitle;

  const DeleteCardSuccess({required this.cardTitle});

  @override
  List<Object?> get props => [cardTitle];
}
