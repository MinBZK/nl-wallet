import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/timeline/timeline_attribute.dart';
import '../../../../domain/usecase/history/get_wallet_timeline_attributes_usecase.dart';

part 'history_overview_event.dart';
part 'history_overview_state.dart';

class HistoryOverviewBloc extends Bloc<HistoryOverviewEvent, HistoryOverviewState> {
  final GetWalletTimelineAttributesUseCase getWalletTimelineAttributesUseCase;

  HistoryOverviewBloc(this.getWalletTimelineAttributesUseCase) : super(HistoryOverviewInitial()) {
    on<HistoryOverviewLoadTriggered>(_onHistoryOverviewLoadTriggered);

    add(const HistoryOverviewLoadTriggered());
  }

  void _onHistoryOverviewLoadTriggered(HistoryOverviewLoadTriggered event, emit) async {
    emit(const HistoryOverviewLoadInProgress());
    try {
      List<TimelineAttribute> attributes = await getWalletTimelineAttributesUseCase.invoke();
      emit(HistoryOverviewLoadSuccess(attributes));
    } catch (error) {
      emit(const HistoryOverviewLoadFailure());
    }
  }
}
