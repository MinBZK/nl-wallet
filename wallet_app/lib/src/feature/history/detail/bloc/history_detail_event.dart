part of 'history_detail_bloc.dart';

abstract class HistoryDetailEvent extends Equatable {
  const HistoryDetailEvent();
}

class HistoryDetailLoadTriggered extends HistoryDetailEvent {
  final TimelineAttribute attribute;
  final String? docType;

  const HistoryDetailLoadTriggered({required this.attribute, required this.docType});

  @override
  List<Object?> get props => [attribute, docType];
}
