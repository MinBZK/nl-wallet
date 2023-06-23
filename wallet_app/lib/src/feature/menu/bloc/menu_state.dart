part of 'menu_bloc.dart';

abstract class MenuState extends Equatable {
  const MenuState();
}

class MenuInitial extends MenuState {
  @override
  List<Object> get props => [];
}

class MenuLoadInProgress extends MenuState {
  const MenuLoadInProgress();

  @override
  List<Object> get props => [];
}

class MenuLoadSuccess extends MenuState {
  final String name;

  const MenuLoadSuccess({required this.name});

  @override
  List<Object> get props => [name];
}
