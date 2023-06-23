part of 'menu_bloc.dart';

abstract class MenuEvent extends Equatable {
  const MenuEvent();

  @override
  List<Object?> get props => [];
}

class MenuLoadTriggered extends MenuEvent {}

class MenuLockWalletPressed extends MenuEvent {}
