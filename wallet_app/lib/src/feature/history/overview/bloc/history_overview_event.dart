part of 'history_overview_bloc.dart';

abstract class HistoryOverviewEvent extends Equatable {
  const HistoryOverviewEvent();

  @override
  List<Object?> get props => [];
}

class HistoryOverviewLoadTriggered extends HistoryOverviewEvent {
  const HistoryOverviewLoadTriggered();
}

class HistoryOverviewLoadNextPageTriggered extends HistoryOverviewEvent {
  const HistoryOverviewLoadNextPageTriggered();
}
