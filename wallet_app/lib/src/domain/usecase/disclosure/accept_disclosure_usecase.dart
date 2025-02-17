import '../pin/check_pin_usecase.dart';

export '../../../data/repository/disclosure/disclosure_repository.dart';

abstract class AcceptDisclosureUseCase extends CheckPinUseCase {
  @override
  Future<Result<String?>> invoke(String pin);
}
