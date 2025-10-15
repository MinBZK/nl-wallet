import '../../../../domain/app_event/app_event_listener.dart';
import '../../../../domain/model/navigation/navigation_request.dart';
import '../../../../domain/model/wallet_status.dart';
import '../../../../domain/usecase/transfer/cancel_wallet_transfer_usecase.dart';
import '../../../../domain/usecase/wallet/get_wallet_status_usecase.dart';
import '../../navigation_service.dart';

class WalletTransferEventListener extends AppEventListener {
  final NavigationService _navigationService;
  final GetWalletStatusUseCase _getWalletStatusUseCase;
  final CancelWalletTransferUseCase _cancelWalletTransferUseCase;

  WalletTransferEventListener(
    this._navigationService,
    this._getWalletStatusUseCase,
    this._cancelWalletTransferUseCase,
  );

  @override
  Future<void> onWalletUnlocked() async {
    final WalletStatus status = await _getWalletStatusUseCase.invoke();
    if (status is! WalletStatusTransferring) return;
    await _cancelWalletTransferUseCase.invoke();
    if (status.role == TransferRole.target) {
      await _navigationService.handleNavigationRequest(
        NavigationRequest.walletTransferTarget(isRetry: true),
        queueIfNotReady: true,
      );
    }
  }
}
