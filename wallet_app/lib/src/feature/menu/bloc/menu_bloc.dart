import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/wallet/lock_wallet_usecase.dart';

part 'menu_event.dart';
part 'menu_state.dart';

class MenuBloc extends Bloc<MenuEvent, MenuState> {
  final LockWalletUseCase lockWalletUseCase;

  MenuBloc(this.lockWalletUseCase) : super(const MenuInitial()) {
    on<MenuLockWalletPressed>(_onLockWalletPressed);
  }

  void _onLockWalletPressed(event, emit) => lockWalletUseCase.invoke();
}
