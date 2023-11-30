part of 'history_detail_bloc.dart';

abstract class HistoryDetailEvent extends Equatable {
  const HistoryDetailEvent();
}

class HistoryDetailLoadTriggered extends HistoryDetailEvent {
  final String attributeId;
  final String? docType;

  const HistoryDetailLoadTriggered({required this.attributeId, required this.docType});

  @override
  List<Object?> get props => [attributeId, docType];
}
