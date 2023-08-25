part of 'card_overview_bloc.dart';

abstract class CardOverviewEvent extends Equatable {
  const CardOverviewEvent();
}

class CardOverviewLoadTriggered extends CardOverviewEvent {
  final bool forceRefresh;

  const CardOverviewLoadTriggered({this.forceRefresh = false});

  @override
  List<Object?> get props => [forceRefresh];
}
