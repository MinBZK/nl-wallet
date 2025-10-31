import '../../model/transfer/transfer_session_state.dart';
import '../wallet_usecase.dart';

/// Use case for observing the [TransferSessionState].
abstract class ObserveTransferSessionStateUseCase extends WalletUseCase {
  Stream<TransferSessionState> invoke();
}
