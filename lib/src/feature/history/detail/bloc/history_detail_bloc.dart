import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/timeline_attribute.dart';
import '../../../../domain/usecase/history/get_timeline_attribute_usecase.dart';

part 'history_detail_event.dart';
part 'history_detail_state.dart';

class HistoryDetailBloc extends Bloc<HistoryDetailEvent, HistoryDetailState> {
  final GetTimelineAttributeUseCase getTimelineAttributeUseCase;

  HistoryDetailBloc(this.getTimelineAttributeUseCase) : super(HistoryDetailInitial()) {
    on<HistoryDetailLoadTriggered>(_onHistoryDetailLoadTriggered);
  }

  void _onHistoryDetailLoadTriggered(HistoryDetailLoadTriggered event, emit) async {
    emit(const HistoryDetailLoadInProgress());
    try {
      TimelineAttribute attribute = await getTimelineAttributeUseCase.invoke(event.attributeId);
      emit(HistoryDetailLoadSuccess(attribute));
    } catch (error) {
      emit(const HistoryDetailLoadFailure());
    }
  }
}
