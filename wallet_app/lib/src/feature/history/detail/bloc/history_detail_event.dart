part of 'history_detail_bloc.dart';

abstract class HistoryDetailEvent extends Equatable {
  const HistoryDetailEvent();
}

class HistoryDetailLoadTriggered extends HistoryDetailEvent {
  final WalletEvent event;
  final String? docType;

  const HistoryDetailLoadTriggered({required this.event, required this.docType});

  @override
  List<Object?> get props => [event, docType];
}
