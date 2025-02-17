import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/event/wallet_event.dart';
import '../../../../domain/model/wallet_card.dart';
import '../../../../domain/usecase/card/get_wallet_card_usecase.dart';
import '../../../../domain/usecase/event/get_wallet_events_for_card_usecase.dart';

part 'card_history_event.dart';
part 'card_history_state.dart';

class CardHistoryBloc extends Bloc<CardHistoryEvent, CardHistoryState> {
  final GetWalletCardUseCase getWalletCardUseCase;
  final GetWalletEventsForCardUseCase getEventsForCardUseCase;

  CardHistoryBloc(
    this.getWalletCardUseCase,
    this.getEventsForCardUseCase,
  ) : super(CardHistoryInitial()) {
    on<CardHistoryLoadTriggered>(_onCardHistoryLoadTriggered);
  }

  Future<void> _onCardHistoryLoadTriggered(CardHistoryLoadTriggered event, emit) async {
    emit(const CardHistoryLoadInProgress());
    final cardResult = await getWalletCardUseCase.invoke(event.docType);
    final eventsResult = await getEventsForCardUseCase.invoke(event.docType);

    if (cardResult.hasError || eventsResult.hasError) {
      emit(const CardHistoryLoadFailure());
    } else {
      emit(CardHistoryLoadSuccess(cardResult.value!, eventsResult.value!));
    }
  }
}
