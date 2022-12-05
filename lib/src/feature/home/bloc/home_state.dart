part of 'home_bloc.dart';

abstract class HomeState extends Equatable {
  final int screenIndex;

  const HomeState(this.screenIndex);
}

class HomeScreenSelect extends HomeState {
  // Used to distinguish between repeated presses
  final int timestamp;

  const HomeScreenSelect(super.screenIndex, {this.timestamp = 0});

  @override
  List<Object> get props => [screenIndex, timestamp];
}
