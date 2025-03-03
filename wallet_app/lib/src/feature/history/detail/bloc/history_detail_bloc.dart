import 'package:collection/collection.dart';
import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/card/wallet_card.dart';
import '../../../../domain/model/event/wallet_event.dart';
import '../../../../domain/usecase/card/get_wallet_cards_usecase.dart';
import '../../../../util/extension/wallet_event_extension.dart';

part 'history_detail_event.dart';
part 'history_detail_state.dart';

class HistoryDetailBloc extends Bloc<HistoryDetailEvent, HistoryDetailState> {
  final GetWalletCardsUseCase getWalletCardsUseCase;

  HistoryDetailBloc(this.getWalletCardsUseCase) : super(HistoryDetailInitial()) {
    on<HistoryDetailLoadTriggered>(_onHistoryDetailLoadTriggered);
  }

  Future<void> _onHistoryDetailLoadTriggered(HistoryDetailLoadTriggered event, emit) async {
    emit(const HistoryDetailLoadInProgress());
    final relatedCardDocTypes = event.event.attributes.map((e) => e.sourceCardDocType).toSet();
    final cardsResult = await getWalletCardsUseCase.invoke();
    await cardsResult.process(
      onSuccess: (cards) {
        final relatedCards = cards.where((card) => relatedCardDocTypes.contains(card.docType));
        emit(HistoryDetailLoadSuccess(event.event, relatedCards.toList()));
      },
      onError: (error) {
        Fimber.e('Failed to load history details', ex: error);
        emit(const HistoryDetailLoadFailure());
      },
    );
  }
}
