import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/service/navigation_service.dart';
import '../../../domain/model/navigation/navigation_request.dart';
import '../../../domain/usecase/wallet/lock_wallet_usecase.dart';

part 'menu_event.dart';
part 'menu_state.dart';

class MenuBloc extends Bloc<MenuEvent, MenuState> {
  final LockWalletUseCase _lockWalletUseCase;
  final NavigationService _navigationService;

  MenuBloc(
    this._lockWalletUseCase,
    this._navigationService,
  ) : super(const MenuInitial()) {
    on<MenuLockWalletPressed>(_onLockWalletPressed);
  }

  Future<void> _onLockWalletPressed(MenuLockWalletPressed event, emit) async {
    await _lockWalletUseCase.invoke();
    await _navigationService.handleNavigationRequest(NavigationRequest.dashboard(), queueIfNotReady: false);
  }
}
