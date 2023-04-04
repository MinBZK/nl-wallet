import '../../model/sign_request.dart';

abstract class GetSignRequestUseCase {
  Future<SignRequest> invoke(String sessionId);
}
