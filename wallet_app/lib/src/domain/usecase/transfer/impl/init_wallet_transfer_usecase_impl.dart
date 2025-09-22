import '../../../../data/repository/transfer/transfer_repository.dart';
import '../../../model/result/result.dart';
import '../init_wallet_transfer_usecase.dart';

class InitWalletTransferUseCaseImpl extends InitWalletTransferUseCase {
  final TransferRepository _transferRepository;

  InitWalletTransferUseCaseImpl(this._transferRepository);

  @override
  Future<Result<String>> invoke() async {
    return tryCatch<String>(
      _transferRepository.initWalletTransfer,
      'Failed to init wallet transfer',
    );
  }
}
