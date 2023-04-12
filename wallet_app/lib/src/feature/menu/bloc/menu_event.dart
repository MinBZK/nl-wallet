part of 'menu_bloc.dart';

abstract class MenuEvent extends Equatable {
  const MenuEvent();

  @override
  List<Object?> get props => [];
}

class MenuLoadTriggered extends MenuEvent {}

class MenuSettingsPressed extends MenuEvent {}

class MenuAboutPressed extends MenuEvent {}

class MenuBackPressed extends MenuEvent {}

class MenuHomePressed extends MenuEvent {}

class MenuLockWalletPressed extends MenuEvent {}
