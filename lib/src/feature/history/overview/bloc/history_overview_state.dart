part of 'history_overview_bloc.dart';

abstract class HistoryOverviewState extends Equatable {
  const HistoryOverviewState();
}

class HistoryOverviewInitial extends HistoryOverviewState {
  @override
  List<Object> get props => [];
}

class HistoryOverviewLoadInProgress extends HistoryOverviewState {
  const HistoryOverviewLoadInProgress();

  @override
  List<Object?> get props => [];
}

class HistoryOverviewLoadSuccess extends HistoryOverviewState {
  final List<TimelineAttribute> attributes;

  const HistoryOverviewLoadSuccess(this.attributes);

  @override
  List<Object> get props => [attributes];
}

class HistoryOverviewLoadFailure extends HistoryOverviewState {
  const HistoryOverviewLoadFailure();

  @override
  List<Object> get props => [];
}
