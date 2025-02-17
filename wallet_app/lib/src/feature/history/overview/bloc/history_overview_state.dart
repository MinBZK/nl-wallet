part of 'history_overview_bloc.dart';

sealed class HistoryOverviewState extends Equatable {
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
  final List<WalletEvent> events;

  const HistoryOverviewLoadSuccess(this.events);

  @override
  List<Object> get props => [events];
}

class HistoryOverviewLoadFailure extends HistoryOverviewState implements ErrorState {
  @override
  final ApplicationError error;

  const HistoryOverviewLoadFailure({required this.error});

  @override
  List<Object> get props => [error];
}
