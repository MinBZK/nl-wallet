import '../../../data/repository/pid/pid_repository.dart';
import '../pin/check_pin_usecase.dart';

export '../../model/pin/check_pin_result.dart';

/// Adds the PID that was issued by [IssuePidForReviewUseCase] to the wallet. Unlike other issued cards this requires
/// the PIN, as the recovery code of the PID is disclosed to the wallet provider when committing.
abstract class CommitIssuedPidUseCase extends CheckPinUseCase {
  @override
  Future<Result<TransferState>> invoke(String pin);
}
