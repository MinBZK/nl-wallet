import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/card/lock_wallet_usecase.dart';
import '../../../domain/usecase/wallet/get_first_name_usecase.dart';

part 'menu_event.dart';
part 'menu_state.dart';

class MenuBloc extends Bloc<MenuEvent, MenuState> {
  final LockWalletUseCase lockWalletUseCase;
  final GetFirstNamesUseCase getFirstNamesUseCase;

  MenuBloc(this.getFirstNamesUseCase, this.lockWalletUseCase) : super(MenuInitial()) {
    on<MenuLoadTriggered>(_onLoadTriggered);
    on<MenuLockWalletPressed>(_onLockWalletPressed);
    on<MenuSettingsPressed>(_onSettingsPressed);
    on<MenuAboutPressed>(_onAboutPressed);
    on<MenuBackPressed>(_onBackPressed);
    on<MenuHomePressed>(_onHomePressed);

    //Immediately start loading when bloc is created.
    add(MenuLoadTriggered());
  }

  void _onLoadTriggered(event, emit) async {
    emit(const MenuLoadInProgress());
    final firstName = await getFirstNamesUseCase.invoke();
    emit(MenuLoadSuccess(name: firstName, menu: SelectedMenu.main));
  }

  void _onSettingsPressed(event, emit) async {
    final state = this.state;
    if (state is MenuLoadSuccess) {
      emit(state.copyWith(menu: SelectedMenu.settings));
    }
  }

  void _onAboutPressed(event, emit) async {
    final state = this.state;
    if (state is MenuLoadSuccess) {
      emit(state.copyWith(menu: SelectedMenu.about));
    }
  }

  void _onBackPressed(event, emit) async {
    final state = this.state;
    if (state is MenuLoadSuccess) {
      emit(state.copyWith(menu: SelectedMenu.main));
    }
  }

  void _onHomePressed(event, emit) async {
    final state = this.state;
    if (state is MenuLoadSuccess) {
      emit(state.copyWith(menu: SelectedMenu.main));
    } else {
      add(MenuLoadTriggered());
    }
  }

  void _onLockWalletPressed(event, emit) async {
    lockWalletUseCase.lock();
  }
}
