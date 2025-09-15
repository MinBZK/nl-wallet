import '../../../model/result/result.dart';
import '../skip_wallet_transfer_usecase.dart';

class SkipWalletTransferUseCaseImpl extends SkipWalletTransferUseCase {
  @override
  Future<Result<void>> invoke() async {
    // TODO(Rob): Implement once core supports skip_wallet_transfer
    return const Result.success(null);
  }
}
