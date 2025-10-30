import '../../model/transfer/transfer_session_state.dart';
import '../wallet_usecase.dart';

/// Use case for observing the status of a wallet transfer.
abstract class GetWalletTransferStatusUseCase extends WalletUseCase {
  Stream<TransferSessionState> invoke();
}
