import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/card/lock_wallet_usecase.dart';
import '../../../domain/usecase/wallet/get_first_names_usecase.dart';

part 'menu_event.dart';
part 'menu_state.dart';

class MenuBloc extends Bloc<MenuEvent, MenuState> {
  final LockWalletUseCase lockWalletUseCase;
  final GetFirstNamesUseCase getFirstNamesUseCase;

  MenuBloc(this.getFirstNamesUseCase, this.lockWalletUseCase) : super(MenuInitial()) {
    on<MenuLoadTriggered>(_onLoadTriggered);
    on<MenuLockWalletPressed>(_onLockWalletPressed);

    //Immediately start loading when bloc is created.
    add(MenuLoadTriggered());
  }

  void _onLoadTriggered(event, emit) async {
    emit(const MenuLoadInProgress());
    final firstName = await getFirstNamesUseCase.invoke();
    emit(MenuLoadSuccess(name: firstName));
  }

  void _onLockWalletPressed(event, emit) async {
    lockWalletUseCase.invoke();
  }
}
