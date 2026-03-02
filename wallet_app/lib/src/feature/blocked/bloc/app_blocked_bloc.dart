import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/wallet_state.dart';
import '../../../domain/usecase/wallet/get_wallet_state_usecase.dart';
import '../../../wallet_core/error/core_error.dart';

part 'app_blocked_event.dart';

part 'app_blocked_state.dart';

class AppBlockedBloc extends Bloc<AppBlockedEvent, AppBlockedState> {
  final GetWalletStateUseCase _getWalletStateUseCase;

  AppBlockedBloc(
    this._getWalletStateUseCase,
  ) : super(AppBlockedInitial()) {
    on<AppBlockedLoadTriggered>(_onRefresh);
  }

  Future<void> _onRefresh(AppBlockedLoadTriggered event, Emitter<AppBlockedState> emit) async {
    emit(AppBlockedInitial());
    await Future.delayed(const Duration(milliseconds: 400)); // Loading is practically instant, UX specified delay

    // Check if wallet was blocked as per user request
    if (event.reason == .userRequest) {
      emit(const AppBlockedByUser());
      return;
    }

    // Verify wallet is really blocked, and emit state accordingly
    try {
      final walletState = await _getWalletStateUseCase.invoke();
      if (walletState is! WalletStateBlocked) throw 'Unexpected state: $walletState';
      emit(AppBlockedByAdmin(walletState));
    } catch (ex) {
      Fimber.e('Wallet not in blocked state', ex: ex);
      emit(const AppBlockedError());
    }
  }
}
