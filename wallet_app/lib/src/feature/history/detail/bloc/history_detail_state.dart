part of 'history_detail_bloc.dart';

sealed class HistoryDetailState extends Equatable {
  const HistoryDetailState();
}

class HistoryDetailInitial extends HistoryDetailState {
  @override
  List<Object> get props => [];
}

class HistoryDetailLoadInProgress extends HistoryDetailState {
  const HistoryDetailLoadInProgress();

  @override
  List<Object?> get props => [];
}

class HistoryDetailLoadSuccess extends HistoryDetailState {
  final WalletEvent event;

  const HistoryDetailLoadSuccess(this.event);

  @override
  List<Object> get props => [event];
}

class HistoryDetailLoadFailure extends HistoryDetailState {
  const HistoryDetailLoadFailure();

  @override
  List<Object> get props => [];
}
