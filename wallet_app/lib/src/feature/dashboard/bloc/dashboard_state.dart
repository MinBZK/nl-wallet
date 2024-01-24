part of 'dashboard_bloc.dart';

sealed class DashboardState extends Equatable {
  const DashboardState();
}

class DashboardStateInitial extends DashboardState {
  const DashboardStateInitial();

  @override
  List<Object> get props => [];
}

class DashboardLoadInProgress extends DashboardState {
  const DashboardLoadInProgress();

  @override
  List<Object> get props => [];
}

class DashboardLoadSuccess extends DashboardState {
  final List<WalletCard> cards;

  const DashboardLoadSuccess(this.cards);

  @override
  List<Object> get props => [cards];
}

class DashboardLoadFailure extends DashboardState {
  const DashboardLoadFailure();

  @override
  List<Object> get props => [];
}
