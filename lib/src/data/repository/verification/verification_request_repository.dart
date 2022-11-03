import '../../../feature/verification/model/verification_request.dart';

abstract class VerificationRequestRepository {
  Future<VerificationRequest> getRequest(String sessionId);
}
