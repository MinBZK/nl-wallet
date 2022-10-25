part of 'home_bloc.dart';

abstract class HomeEvent extends Equatable {
  const HomeEvent();
}

class HomeTabPressed extends HomeEvent {
  final int index;

  const HomeTabPressed(this.index);

  @override
  List<Object?> get props => [index];
}
