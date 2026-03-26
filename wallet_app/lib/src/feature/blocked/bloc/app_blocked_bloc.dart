import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/wallet_state.dart';
import '../../../domain/usecase/wallet/get_wallet_state_usecase.dart';
import '../../../wallet_core/error/core_error.dart';

part 'app_blocked_event.dart';

part 'app_blocked_state.dart';

/// Business logic component for the App Blocked screen.
///
/// This BLoC is responsible for determining the specific reason why the app is blocked
/// and emitting the corresponding [AppBlockedState].
class AppBlockedBloc extends Bloc<AppBlockedEvent, AppBlockedState> {
  final GetWalletStateUseCase _getWalletStateUseCase;

  AppBlockedBloc(
    this._getWalletStateUseCase,
  ) : super(AppBlockedInitial()) {
    on<AppBlockedLoadTriggered>(_onRefresh);
  }

  /// Handles the [AppBlockedLoadTriggered] event.
  ///
  /// This method checks if the block was requested by the user or if it's an administrative block
  /// by querying the current [WalletState].
  Future<void> _onRefresh(AppBlockedLoadTriggered event, Emitter<AppBlockedState> emit) async {
    emit(AppBlockedInitial());
    // Artificial delay for better UX, as the check is usually near-instant.
    await Future.delayed(const Duration(milliseconds: 400));

    // Handle the provided reason, crucial because some reasons are transient.
    switch (event.reason) {
      case RevocationReason.userRequest:
        emit(const AppBlockedByUser());
        return; // WalletState is already [WalletStateEmpty], so important to return immediately.
      case RevocationReason.solutionCompromised:
        emit(const AppBlockedSolutionCompromised());
        return;
      case RevocationReason.adminRequest:
      case RevocationReason.unknown:
        Fimber.i('Unable to resolve solely from reason (${event.reason}), resolving from state.');
    }

    // Verify wallet is really blocked, and emit state accordingly
    try {
      final walletState = await _getWalletStateUseCase.invoke();
      if (walletState is! WalletStateBlocked) throw 'Unexpected state: $walletState';
      switch (walletState.reason) {
        case BlockedReason.requiresAppUpdate:
          throw 'Reason ${walletState.reason} should have been caught by [UpdateChecker]';
        case BlockedReason.blockedByWalletProvider:
          emit(AppBlockedByAdmin(walletState));
        case BlockedReason.solutionRevoked:
          emit(const AppBlockedSolutionCompromised());
      }
    } catch (ex) {
      Fimber.e('Wallet not in blocked state', ex: ex);
      emit(const AppBlockedError());
    }
  }
}
