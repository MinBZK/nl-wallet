import '../pin/check_pin_usecase.dart';

abstract class GetRevocationCodeUseCase extends CheckPinUseCase {
  @override
  Future<Result<String>> invoke(String pin);
}
