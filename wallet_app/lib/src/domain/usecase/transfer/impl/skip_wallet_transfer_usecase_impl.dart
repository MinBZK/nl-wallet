import '../../../../data/repository/transfer/transfer_repository.dart';
import '../../../model/result/result.dart';
import '../skip_wallet_transfer_usecase.dart';

class SkipWalletTransferUseCaseImpl extends SkipWalletTransferUseCase {
  final TransferRepository _transferRepository;

  SkipWalletTransferUseCaseImpl(this._transferRepository);

  @override
  Future<Result<void>> invoke() => tryCatch(
    _transferRepository.skipWalletTransfer,
    'Failed to skip transfer',
  );
}
