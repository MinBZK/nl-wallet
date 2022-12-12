part of 'history_detail_bloc.dart';

abstract class HistoryDetailEvent extends Equatable {
  const HistoryDetailEvent();
}

class HistoryDetailLoadTriggered extends HistoryDetailEvent {
  final String attributeId;

  const HistoryDetailLoadTriggered(this.attributeId);

  @override
  List<Object?> get props => [attributeId];
}
