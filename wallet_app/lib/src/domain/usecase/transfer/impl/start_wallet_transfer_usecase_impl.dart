import '../../../model/result/result.dart';
import '../start_wallet_transfer_usecase.dart';

class StartWalletTransferUseCaseImpl extends StartWalletTransferUseCase {
  @override
  Future<Result<void>> invoke(String pin) async {
    // TODO(Rob): Implement once core supports start_wallet_transfer
    await Future.delayed(const Duration(seconds: 1));
    return const Result.success(null);
  }
}
