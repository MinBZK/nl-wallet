import '../../../model/result/result.dart';
import '../cancel_wallet_transfer_usecase.dart';

class CancelWalletTransferUseCaseImpl extends CancelWalletTransferUseCase {
  @override
  Future<Result<void>> invoke() async {
    // TODO(Rob): Implement once core supports cancel_wallet_transfer
    return const Result.success(null);
  }
}
