import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

part 'delete_card_event.dart';
part 'delete_card_state.dart';

class DeleteCardBloc extends Bloc<DeleteCardEvent, DeleteCardState> {
  DeleteCardBloc() : super(const DeleteCardInitial()) {
    on<DeleteCardLoadTriggered>(_onLoadTriggered);
    on<DeleteCardPinConfirmed>(_onPinConfirmed);
  }

  FutureOr<void> _onLoadTriggered(DeleteCardLoadTriggered event, emit) async {
    emit(DeleteCardProvidePin(attestationId: event.attestationId, cardTitle: event.cardTitle));
  }

  FutureOr<void> _onPinConfirmed(DeleteCardPinConfirmed event, emit) async {
    final currentState = state;
    if (currentState is DeleteCardProvidePin) {
      emit(DeleteCardSuccess(cardTitle: currentState.cardTitle));
    }
  }
}
