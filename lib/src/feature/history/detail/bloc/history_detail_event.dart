part of 'history_detail_bloc.dart';

abstract class HistoryDetailEvent extends Equatable {
  const HistoryDetailEvent();
}

class HistoryDetailLoadTriggered extends HistoryDetailEvent {
  final String attributeId;
  final String? cardId;

  const HistoryDetailLoadTriggered({required this.attributeId, required this.cardId});

  @override
  List<Object?> get props => [attributeId, cardId];
}
