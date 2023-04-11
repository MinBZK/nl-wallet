part of 'history_overview_bloc.dart';

abstract class HistoryOverviewEvent extends Equatable {
  const HistoryOverviewEvent();
}

class HistoryOverviewLoadTriggered extends HistoryOverviewEvent {
  const HistoryOverviewLoadTriggered();

  @override
  List<Object?> get props => [];
}
