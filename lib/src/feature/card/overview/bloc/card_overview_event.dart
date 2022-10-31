part of 'card_overview_bloc.dart';

abstract class CardOverviewEvent extends Equatable {
  const CardOverviewEvent();
}

class CardOverviewLoadTriggered extends CardOverviewEvent {
  @override
  List<Object?> get props => [];
}

class CardOverviewLockWalletPressed extends CardOverviewEvent {
  @override
  List<Object?> get props => [];
}
