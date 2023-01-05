part of 'home_bloc.dart';

abstract class HomeEvent extends Equatable {
  const HomeEvent();
}

class HomeTabPressed extends HomeEvent {
  final int tabIndex;

  const HomeTabPressed(this.tabIndex);

  HomeScreenTab get tab => HomeScreenTabExtension.from(tabIndex);

  @override
  List<Object?> get props => [tabIndex];
}
