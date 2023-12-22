import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/timeline/timeline_attribute.dart';
import '../../../../domain/model/wallet_card.dart';
import '../../../../domain/usecase/card/get_wallet_card_timeline_attributes_usecase.dart';
import '../../../../domain/usecase/card/get_wallet_card_usecase.dart';

part 'card_history_event.dart';
part 'card_history_state.dart';

class CardHistoryBloc extends Bloc<CardHistoryEvent, CardHistoryState> {
  final GetWalletCardUseCase getWalletCardUseCase;
  final GetWalletCardTimelineAttributesUseCase getWalletCardTimelineAttributesUseCase;

  CardHistoryBloc(
    this.getWalletCardUseCase,
    this.getWalletCardTimelineAttributesUseCase,
  ) : super(CardHistoryInitial()) {
    on<CardHistoryLoadTriggered>(_onCardHistoryLoadTriggered);
  }

  void _onCardHistoryLoadTriggered(CardHistoryLoadTriggered event, emit) async {
    emit(const CardHistoryLoadInProgress());
    try {
      WalletCard card = await getWalletCardUseCase.invoke(event.docType);
      List<TimelineAttribute> attributes = await getWalletCardTimelineAttributesUseCase.invoke(event.docType);
      emit(CardHistoryLoadSuccess(card, attributes));
    } catch (error, stack) {
      Fimber.e('Failed to load card history for ${event.docType}', ex: error, stacktrace: stack);
      emit(const CardHistoryLoadFailure());
    }
  }
}
