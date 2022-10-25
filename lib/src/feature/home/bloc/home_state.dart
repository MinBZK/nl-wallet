part of 'home_bloc.dart';

abstract class HomeState extends Equatable {
  final int screenIndex;

  const HomeState(this.screenIndex);
}

class HomeScreenSelect extends HomeState {
  const HomeScreenSelect(super.screenIndex);

  @override
  List<Object> get props => [screenIndex];
}
