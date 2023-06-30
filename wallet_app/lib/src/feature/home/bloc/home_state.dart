part of 'home_bloc.dart';

sealed class HomeState extends Equatable {
  final HomeTab tab;

  const HomeState(this.tab);
}

class HomeScreenSelect extends HomeState {
  // Can be set to make the State unique, to make sure the UI rebuilds.
  final int? uid;

  const HomeScreenSelect(super.tab, {this.uid});

  @override
  List<Object?> get props => [tab, uid];
}

enum HomeTab { cards, qr, menu }
