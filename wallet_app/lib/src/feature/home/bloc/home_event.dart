part of 'home_bloc.dart';

abstract class HomeEvent extends Equatable {
  const HomeEvent();
}

class HomeTabPressed extends HomeEvent {
  final HomeTab tab;
  final bool forceStateRefresh;

  const HomeTabPressed(this.tab, {this.forceStateRefresh = false});

  @override
  List<Object?> get props => [tab, forceStateRefresh];
}
