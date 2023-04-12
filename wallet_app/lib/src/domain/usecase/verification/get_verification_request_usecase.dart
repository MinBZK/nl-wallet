import '../../../feature/verification/model/verification_request.dart';

abstract class GetVerificationRequestUseCase {
  Future<VerificationRequest> invoke(String sessionId);
}
