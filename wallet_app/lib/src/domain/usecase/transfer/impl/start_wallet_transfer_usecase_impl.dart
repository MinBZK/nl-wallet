import '../../../../data/repository/transfer/transfer_repository.dart';
import '../../../model/result/result.dart';
import '../start_wallet_transfer_usecase.dart';

class StartWalletTransferUseCaseImpl extends StartWalletTransferUseCase {
  final TransferRepository _transferRepository;

  StartWalletTransferUseCaseImpl(this._transferRepository);

  @override
  Future<Result<void>> invoke() {
    return tryCatch(
      _transferRepository.transferWallet,
      'Failed to transfer wallet',
    );
  }
}
