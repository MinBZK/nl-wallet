import '../../../../data/repository/transfer/transfer_repository.dart';
import '../../../model/result/result.dart';
import '../pair_wallet_transfer_usecase.dart';

class PairWalletTransferUseCaseImpl extends PairWalletTransferUseCase {
  final TransferRepository _transferRepository;

  PairWalletTransferUseCaseImpl(this._transferRepository);

  @override
  Future<Result<void>> invoke(String uri) async {
    return tryCatch(
      () async => _transferRepository.pairWalletTransfer(uri),
      'Failed to acknowledge wallet transfer',
    );
  }
}
