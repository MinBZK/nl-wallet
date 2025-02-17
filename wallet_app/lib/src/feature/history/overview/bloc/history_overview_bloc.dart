import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/bloc/error_state.dart';
import '../../../../domain/model/event/wallet_event.dart';
import '../../../../domain/model/result/application_error.dart';
import '../../../../domain/usecase/event/get_wallet_events_usecase.dart';

part 'history_overview_event.dart';
part 'history_overview_state.dart';

class HistoryOverviewBloc extends Bloc<HistoryOverviewEvent, HistoryOverviewState> {
  final GetWalletEventsUseCase getWalletEventsUseCase;

  HistoryOverviewBloc(this.getWalletEventsUseCase) : super(HistoryOverviewInitial()) {
    on<HistoryOverviewLoadTriggered>(_onHistoryOverviewLoadTriggered);
  }

  Future<void> _onHistoryOverviewLoadTriggered(HistoryOverviewLoadTriggered event, emit) async {
    emit(const HistoryOverviewLoadInProgress());
    final eventsResult = await getWalletEventsUseCase.invoke();
    await eventsResult.process(
      onSuccess: (events) => emit(HistoryOverviewLoadSuccess(events)),
      onError: (error) => emit(HistoryOverviewLoadFailure(error: error)),
    );
  }
}
