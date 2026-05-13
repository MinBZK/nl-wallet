import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/bloc/error_state.dart';
import '../../../../domain/model/event/wallet_event.dart';
import '../../../../domain/model/result/application_error.dart';

part 'history_detail_event.dart';
part 'history_detail_state.dart';

// HistoryDetailLoadFailure is kept for future use (i.e. when async loading is reintroduced)
// and to keep the error rendering path covered by goldens.
class HistoryDetailBloc extends Bloc<HistoryDetailEvent, HistoryDetailState> {
  HistoryDetailBloc() : super(const HistoryDetailInitial()) {
    on<HistoryDetailLoadTriggered>(_onHistoryDetailLoadTriggered);
  }

  Future<void> _onHistoryDetailLoadTriggered(HistoryDetailLoadTriggered event, emit) async {
    // Currently not actively loading data, skipping emission of LoadInProgress/LoadFailure
    emit(HistoryDetailLoadSuccess(event.event));
  }
}
