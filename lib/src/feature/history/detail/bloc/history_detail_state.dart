part of 'history_detail_bloc.dart';

abstract class HistoryDetailState extends Equatable {
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
  final TimelineAttribute attribute;

  const HistoryDetailLoadSuccess(this.attribute);

  @override
  List<Object> get props => [attribute];
}

class HistoryDetailLoadFailure extends HistoryDetailState {
  const HistoryDetailLoadFailure();

  @override
  List<Object> get props => [];
}
