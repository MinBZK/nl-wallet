import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/timeline_attribute.dart';
import '../../../../domain/usecase/card/get_wallet_card_timeline_usecase.dart';

part 'card_history_event.dart';
part 'card_history_state.dart';

class CardHistoryBloc extends Bloc<CardHistoryEvent, CardHistoryState> {
  final GetWalletCardTimelineUseCase getWalletCardTimelineUseCase;

  CardHistoryBloc(this.getWalletCardTimelineUseCase) : super(CardHistoryInitial()) {
    on<CardHistoryLoadTriggered>(_onCardHistoryLoadTriggered);
  }

  void _onCardHistoryLoadTriggered(CardHistoryLoadTriggered event, emit) async {
    emit(const CardHistoryLoadInProgress());
    try {
      List<TimelineAttribute> results = await getWalletCardTimelineUseCase.getAll(event.cardId);
      emit(CardHistoryLoadSuccess(results));
    } catch (error) {
      emit(const CardHistoryLoadFailure());
    }
  }
}
