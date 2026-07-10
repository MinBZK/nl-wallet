import 'dart:collection';

import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:collection/collection.dart';
import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/bloc/error_state.dart';
import '../../../../domain/model/event/wallet_event.dart';
import '../../../../domain/model/result/application_error.dart';
import '../../../../domain/usecase/event/get_wallet_events_page_usecase.dart';
import '../../../../util/extension/date_time_extension.dart';

part 'history_overview_event.dart';
part 'history_overview_state.dart';

const int _kHistoryPageSize = 25;

class HistoryOverviewBloc extends Bloc<HistoryOverviewEvent, HistoryOverviewState> {
  final GetWalletEventsPageUseCase _getWalletEventsPageUseCase;

  HistoryOverviewBloc(this._getWalletEventsPageUseCase) : super(const HistoryOverviewInitial()) {
    on<HistoryOverviewLoadTriggered>(_onLoadTriggered);
    on<HistoryOverviewLoadNextPageTriggered>(_onLoadNextPageTriggered, transformer: droppable());
  }

  Future<void> _onLoadTriggered(HistoryOverviewLoadTriggered event, emit) async {
    emit(const HistoryOverviewLoadInProgress());
    final result = await _getWalletEventsPageUseCase.invoke(
      page: 0,
      pageSize: _kHistoryPageSize,
      currentPages: SplayTreeMap(),
    );
    await result.process(
      onSuccess: (page) => emit(
        HistoryOverviewLoadSuccess(pages: page.pages, lastLoadedPage: 0, hasNextPage: page.hasNextPage),
      ),
      onError: (error) => emit(HistoryOverviewLoadFailure(error: error)),
    );
  }

  Future<void> _onLoadNextPageTriggered(HistoryOverviewLoadNextPageTriggered event, emit) async {
    final currentState = state;
    if (currentState is! HistoryOverviewLoadSuccess) return;
    if (!currentState.hasNextPage || currentState.isLoadingMore) return;

    emit(currentState.copyWith(isLoadingMore: true));

    final nextPage = currentState.lastLoadedPage + 1;
    final result = await _getWalletEventsPageUseCase.invoke(
      page: nextPage,
      pageSize: _kHistoryPageSize,
      currentPages: currentState.pages,
    );
    await result.process(
      onSuccess: (page) => emit(
        HistoryOverviewLoadSuccess(pages: page.pages, lastLoadedPage: nextPage, hasNextPage: page.hasNextPage),
      ),
      onError: (_) => emit(currentState.copyWith(isLoadingMore: false)),
    );
  }
}
