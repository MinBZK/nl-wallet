import 'dart:async';

import '../../../../domain/app_event/app_event_listener.dart';
import '../../../../domain/model/wallet_state.dart';
import '../../../../domain/usecase/transfer/cancel_wallet_transfer_usecase.dart';
import '../../../../domain/usecase/wallet/get_wallet_state_usecase.dart';
import '../../navigation_service.dart';

/// [AppEventListener] that observes events that require transfer related actions
class WalletTransferAppEventListener extends AppEventListener {
  final NavigationService _navigationService;
  final GetWalletStateUseCase _getWalletStateUseCase;
  final CancelWalletTransferUseCase _cancelWalletTransferUseCase;

  WalletTransferAppEventListener(
    this._navigationService,
    this._getWalletStateUseCase,
    this._cancelWalletTransferUseCase,
  );

  @override
  Future<void> onWalletUnlocked() async {
    final WalletState state = await _getWalletStateUseCase.invoke();
    if (state is WalletStateTransferring) {
      await _cancelWalletTransferUseCase.invoke();
      if (state.role == .destination) unawaited(_navigationService.showDialog(.moveStopped));
    }
  }

  @override
  Future<void> onDashboardShown() async {
    final WalletState state = await _getWalletStateUseCase.invoke();
    if (state is WalletStateTransferring && state.role == TransferRole.source) {
      await _cancelWalletTransferUseCase.invoke();
    }
  }
}
