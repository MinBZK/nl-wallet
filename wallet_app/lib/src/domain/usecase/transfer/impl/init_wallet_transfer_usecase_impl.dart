import '../../../model/result/result.dart';
import '../init_wallet_transfer_usecase.dart';

class InitWalletTransferUseCaseImpl extends InitWalletTransferUseCase {
  @override
  Future<Result<void>> invoke() async {
    // TODO(Rob): Implement once core supports init_wallet_transfer
    await Future.delayed(const Duration(seconds: 1));
    return const Result.success(null);
  }
}
