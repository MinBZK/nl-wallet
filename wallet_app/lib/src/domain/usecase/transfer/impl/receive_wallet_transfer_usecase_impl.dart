import '../../../../data/repository/transfer/transfer_repository.dart';
import '../../../model/result/result.dart';
import '../receive_wallet_transfer_usecase.dart';

class ReceiveWalletTransferUseCaseImpl extends ReceiveWalletTransferUseCase {
  final TransferRepository _transferRepository;

  ReceiveWalletTransferUseCaseImpl(this._transferRepository);

  @override
  Future<Result<void>> invoke() => tryCatch(
    _transferRepository.receiveWalletTransfer,
    'Failed to receive transfer',
  );
}
