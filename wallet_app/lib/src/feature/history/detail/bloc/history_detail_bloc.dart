import 'package:collection/collection.dart';
import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../domain/model/timeline/timeline_attribute.dart';
import '../../../../domain/model/wallet_card.dart';
import '../../../../domain/usecase/card/get_wallet_card_usecase.dart';
import '../../../../domain/usecase/history/get_timeline_attribute_usecase.dart';

part 'history_detail_event.dart';
part 'history_detail_state.dart';

class HistoryDetailBloc extends Bloc<HistoryDetailEvent, HistoryDetailState> {
  final GetTimelineAttributeUseCase getTimelineAttributeUseCase;
  final GetWalletCardUseCase getWalletCardUseCase;

  HistoryDetailBloc(this.getTimelineAttributeUseCase, this.getWalletCardUseCase) : super(HistoryDetailInitial()) {
    on<HistoryDetailLoadTriggered>(_onHistoryDetailLoadTriggered);
  }

  void _onHistoryDetailLoadTriggered(HistoryDetailLoadTriggered event, emit) async {
    emit(const HistoryDetailLoadInProgress());
    try {
      TimelineAttribute timelineAttribute = await getTimelineAttributeUseCase.invoke(
        timelineAttributeId: event.attributeId,
        cardId: event.cardId,
      );
      final relatedCardIds = timelineAttribute.dataAttributes.map((e) => e.sourceCardId).toSet();
      final relatedCardFutures = relatedCardIds.map((cardId) => getWalletCardUseCase.invoke(cardId));
      final relatedCards = await Future.wait(relatedCardFutures);
      emit(HistoryDetailLoadSuccess(timelineAttribute, relatedCards));
    } catch (error) {
      emit(const HistoryDetailLoadFailure());
    }
  }
}
