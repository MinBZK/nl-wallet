import 'package:collection/collection.dart';
import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../domain/model/timeline/timeline_attribute.dart';
import '../../../../domain/model/wallet_card.dart';
import '../../../../domain/usecase/card/get_wallet_cards_usecase.dart';
import '../../../../domain/usecase/history/get_timeline_attribute_usecase.dart';

part 'history_detail_event.dart';
part 'history_detail_state.dart';

class HistoryDetailBloc extends Bloc<HistoryDetailEvent, HistoryDetailState> {
  final GetTimelineAttributeUseCase getTimelineAttributeUseCase;
  final GetWalletCardsUseCase getWalletCardsUseCase;

  HistoryDetailBloc(this.getTimelineAttributeUseCase, this.getWalletCardsUseCase) : super(HistoryDetailInitial()) {
    on<HistoryDetailLoadTriggered>(_onHistoryDetailLoadTriggered);
  }

  void _onHistoryDetailLoadTriggered(HistoryDetailLoadTriggered event, emit) async {
    emit(const HistoryDetailLoadInProgress());
    try {
      TimelineAttribute timelineAttribute = await getTimelineAttributeUseCase.invoke(
        timelineAttributeId: event.attributeId,
        docType: event.docType,
      );
      final relatedCardDocTypes = timelineAttribute.dataAttributes.map((e) => e.sourceCardDocType).toSet();
      final relatedCards =
          (await getWalletCardsUseCase.invoke()).where((card) => relatedCardDocTypes.contains(card.docType));
      emit(HistoryDetailLoadSuccess(timelineAttribute, relatedCards.toList()));
    } catch (error) {
      Fimber.e('Failed to load history details', ex: error);
      emit(const HistoryDetailLoadFailure());
    }
  }
}
