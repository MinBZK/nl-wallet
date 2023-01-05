part of 'home_bloc.dart';

abstract class HomeState extends Equatable {
  final HomeScreenTab tab;

  const HomeState(this.tab);
}

class HomeScreenSelect extends HomeState {
  // Used to distinguish between repeated presses
  final int timestamp;

  const HomeScreenSelect(super.tab, {this.timestamp = 0});

  @override
  List<Object> get props => [tab, timestamp];
}

enum HomeScreenTab { cards, qr, menu }

extension HomeScreenTabExtension on HomeScreenTab {
  int get tabIndex {
    switch (this) {
      case HomeScreenTab.cards:
        return 0;
      case HomeScreenTab.qr:
        return 1;
      case HomeScreenTab.menu:
        return 2;
    }
  }

  static HomeScreenTab from(int tabIndex) {
    switch (tabIndex) {
      case 0:
        return HomeScreenTab.cards;
      case 1:
        return HomeScreenTab.qr;
      case 2:
        return HomeScreenTab.menu;
    }
    throw UnsupportedError('Unknown HomeScreenTab index: $tabIndex');
  }
}
