import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/event/wallet_event.dart';
import '../../../../domain/usecase/event/get_wallet_events_usecase.dart';

part 'history_overview_event.dart';
part 'history_overview_state.dart';

class HistoryOverviewBloc extends Bloc<HistoryOverviewEvent, HistoryOverviewState> {
  final GetWalletEventsUseCase getWalletEventsUseCase;

  HistoryOverviewBloc(this.getWalletEventsUseCase) : super(HistoryOverviewInitial()) {
    on<HistoryOverviewLoadTriggered>(_onHistoryOverviewLoadTriggered);

    add(const HistoryOverviewLoadTriggered());
  }

  void _onHistoryOverviewLoadTriggered(HistoryOverviewLoadTriggered event, emit) async {
    emit(const HistoryOverviewLoadInProgress());
    try {
      List<WalletEvent> attributes = await getWalletEventsUseCase.invoke();
      emit(HistoryOverviewLoadSuccess(attributes));
    } catch (error) {
      Fimber.e('Failed to load history', ex: error);
      emit(const HistoryOverviewLoadFailure());
    }
  }
}
