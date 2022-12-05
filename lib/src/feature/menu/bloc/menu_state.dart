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
  final SelectedMenu menu;

  const MenuLoadSuccess({required this.name, required this.menu});

  @override
  List<Object> get props => [name, menu];

  MenuLoadSuccess copyWith({
    String? name,
    SelectedMenu? menu,
  }) {
    return MenuLoadSuccess(
      name: name ?? this.name,
      menu: menu ?? this.menu,
    );
  }
}

enum SelectedMenu { main, settings, about }
