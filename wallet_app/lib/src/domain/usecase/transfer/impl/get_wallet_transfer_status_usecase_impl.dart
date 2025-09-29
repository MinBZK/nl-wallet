import '../../../../data/repository/transfer/transfer_repository.dart';
import '../../../model/transfer/wallet_transfer_status.dart';
import '../get_wallet_transfer_status_usecase.dart';

/// Use case for observing the status of a wallet transfer.
///
/// This class polls the [TransferRepository] for the current transfer status
/// and yields the status until a terminal state is reached.
class GetWalletTransferStatusUseCaseImpl extends GetWalletTransferStatusUseCase {
  final TransferRepository _transferRepository;

  static const List<WalletTransferStatus> _terminalStates = [
    WalletTransferStatus.success,
    WalletTransferStatus.cancelled,
    WalletTransferStatus.error,
  ];

  GetWalletTransferStatusUseCaseImpl(this._transferRepository);

  @override
  Stream<WalletTransferStatus> invoke() async* {
    while (true) {
      final status = await _transferRepository.getWalletTransferState();
      yield status;
      if (_terminalStates.contains(status)) return;
      await Future.delayed(const Duration(seconds: 3));
    }
  }
}
