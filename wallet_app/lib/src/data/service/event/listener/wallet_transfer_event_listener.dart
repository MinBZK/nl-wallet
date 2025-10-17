import '../../../../domain/app_event/app_event_listener.dart';
import '../../../../domain/model/navigation/navigation_request.dart';
import '../../../../domain/model/wallet_state.dart';
import '../../../../domain/usecase/transfer/cancel_wallet_transfer_usecase.dart';
import '../../../../domain/usecase/wallet/get_wallet_state_usecase.dart';
import '../../navigation_service.dart';

class WalletTransferEventListener extends AppEventListener {
  final NavigationService _navigationService;
  final GetWalletStateUseCase _getWalletStateUseCase;
  final CancelWalletTransferUseCase _cancelWalletTransferUseCase;

  WalletTransferEventListener(
    this._navigationService,
    this._getWalletStateUseCase,
    this._cancelWalletTransferUseCase,
  );

  @override
  Future<void> onWalletUnlocked() async {
    final WalletState status = await _getWalletStateUseCase.invoke();
    switch (status) {
      case WalletStateTransferPossible():
        await _navigationService.handleNavigationRequest(
          NavigationRequest.walletTransferTarget(isRetry: false),
          queueIfNotReady: true,
        );
      case WalletStateTransferring():
        await _cancelWalletTransferUseCase.invoke();
        if (status.role == TransferRole.target) {
          await _navigationService.handleNavigationRequest(
            NavigationRequest.walletTransferTarget(isRetry: true),
            queueIfNotReady: true,
          );
        }
      default:
        break;
    }
  }
}
