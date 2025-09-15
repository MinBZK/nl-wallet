import '../../../model/result/result.dart';
import '../prepare_wallet_transfer_usecase.dart';

class PrepareWalletTransferUseCaseImpl extends PrepareWalletTransferUseCase {
  @override
  Future<Result<String>> invoke() async {
    return const Result.success('qr-code-contents-will-go-here');
  }
}
