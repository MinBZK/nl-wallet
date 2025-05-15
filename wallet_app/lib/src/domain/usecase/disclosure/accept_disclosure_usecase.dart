import '../pin/check_pin_usecase.dart';

export '../../../data/repository/disclosure/disclosure_repository.dart';

/// Accept disclosure request, optionally providing a returnUrl
/// to redirect the user after successful disclosure.
abstract class AcceptDisclosureUseCase extends CheckPinUseCase {
  @override
  Future<Result<String?>> invoke(String pin);
}
