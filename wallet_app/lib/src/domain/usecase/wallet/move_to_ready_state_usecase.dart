import '../../model/result/result.dart';
import '../wallet_usecase.dart';

/// Moves the wallet into the ready state by cancelling any ongoing
/// session. Returns true if the wallet is now in the ready state.
///
/// This will never succeed if the wallet is not initialized with pid.
abstract class MoveToReadyStateUseCase extends WalletUseCase {
  Future<Result<bool>> invoke();
}
