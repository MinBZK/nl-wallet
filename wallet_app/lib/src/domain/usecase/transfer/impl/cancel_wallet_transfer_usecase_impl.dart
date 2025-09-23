import '../../../../data/repository/transfer/transfer_repository.dart';
import '../../../model/result/result.dart';
import '../cancel_wallet_transfer_usecase.dart';

class CancelWalletTransferUseCaseImpl extends CancelWalletTransferUseCase {
  final TransferRepository _transferRepository;

  CancelWalletTransferUseCaseImpl(this._transferRepository);

  @override
  Future<Result<void>> invoke() async {
    return tryCatch(
      _transferRepository.cancelWalletTransfer,
      'Failed to cancel wallet transfer',
    );
  }
}
