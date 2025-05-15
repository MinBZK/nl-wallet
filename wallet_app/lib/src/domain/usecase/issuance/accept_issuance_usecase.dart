import '../pin/check_pin_usecase.dart';

/// Accept issuance request (thereby adding cards to wallet), optionally
/// providing a returnUrl to redirect the user after successful disclosure.
abstract class AcceptIssuanceUseCase extends CheckPinUseCase {
  @override
  Future<Result<void>> invoke(String pin);
}
