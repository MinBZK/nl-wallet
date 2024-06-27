part of 'history_detail_bloc.dart';

abstract class HistoryDetailEvent extends Equatable {
  const HistoryDetailEvent();
}

class HistoryDetailLoadTriggered extends HistoryDetailEvent {
  final WalletEvent event;

  const HistoryDetailLoadTriggered({required this.event});

  @override
  List<Object?> get props => [event];
}
