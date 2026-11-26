import '../../model/attribute/attribute.dart';
import '../pin/check_pin_usecase.dart';

export '../../model/pin/check_pin_result.dart';

/// Requests the PID from an issuer that does not provide previews, returns the attributes of the issued PID for review.
/// The PID is not added to the wallet until the user accepts it, see [CommitIssuedPidUseCase].
abstract class IssuePidForReviewUseCase extends CheckPinUseCase {
  @override
  Future<Result<List<Attribute>>> invoke(String pin);
}
