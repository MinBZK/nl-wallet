import 'package:collection/collection.dart';
import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../domain/model/timeline/timeline_attribute.dart';
import '../../../../domain/model/wallet_card.dart';
import '../../../../domain/usecase/card/get_wallet_cards_usecase.dart';

part 'history_detail_event.dart';
part 'history_detail_state.dart';

class HistoryDetailBloc extends Bloc<HistoryDetailEvent, HistoryDetailState> {
  final GetWalletCardsUseCase getWalletCardsUseCase;

  HistoryDetailBloc(this.getWalletCardsUseCase) : super(HistoryDetailInitial()) {
    on<HistoryDetailLoadTriggered>(_onHistoryDetailLoadTriggered);
  }

  void _onHistoryDetailLoadTriggered(HistoryDetailLoadTriggered event, emit) async {
    emit(const HistoryDetailLoadInProgress());
    try {
      final relatedCardDocTypes = event.attribute.dataAttributes.map((e) => e.sourceCardDocType).toSet();
      final relatedCards =
          (await getWalletCardsUseCase.invoke()).where((card) => relatedCardDocTypes.contains(card.docType));
      emit(HistoryDetailLoadSuccess(event.attribute, relatedCards.toList()));
    } catch (error) {
      Fimber.e('Failed to load history details', ex: error);
      emit(const HistoryDetailLoadFailure());
    }
  }
}
