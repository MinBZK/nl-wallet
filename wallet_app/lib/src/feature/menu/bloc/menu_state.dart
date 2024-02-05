part of 'menu_bloc.dart';

sealed class MenuState extends Equatable {
  const MenuState();
}

class MenuInitial extends MenuState {
  const MenuInitial();

  @override
  List<Object> get props => [];
}
