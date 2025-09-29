import '../../../../data/repository/transfer/transfer_repository.dart';
import '../../../model/result/result.dart';
import '../acknowledge_wallet_transfer_usecase.dart';

class AcknowledgeWalletTransferUseCaseImpl extends AcknowledgeWalletTransferUseCase {
  final TransferRepository _transferRepository;

  AcknowledgeWalletTransferUseCaseImpl(this._transferRepository);

  @override
  Future<Result<void>> invoke(String uri) async {
    return tryCatch(
      () async => _transferRepository.acknowledgeWalletTransfer(uri),
      'Failed to acknowledge wallet transfer',
    );
  }
}
