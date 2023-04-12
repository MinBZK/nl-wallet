part of 'home_bloc.dart';

abstract class HomeState extends Equatable {
  final HomeTab tab;

  const HomeState(this.tab);
}

class HomeScreenSelect extends HomeState {
  // Used to distinguish between repeated presses (a.k.a. force state refresh)
  final bool stateToggle;

  const HomeScreenSelect(super.tab, {this.stateToggle = false});

  @override
  List<Object?> get props => [tab, stateToggle];
}

enum HomeTab { cards, qr, menu }

extension HomeTabExtension on HomeTab {
  int get tabIndex {
    switch (this) {
      case HomeTab.cards:
        return 0;
      case HomeTab.qr:
        return 1;
      case HomeTab.menu:
        return 2;
    }
  }
}
