import '../../../data/repository/verification/verification_request_repository.dart';
import '../../../feature/verification/model/verification_request.dart';
import '../../../wallet_constants.dart';

class GetVerificationRequestUseCase {
  final VerificationRequestRepository verificationRepository;

  GetVerificationRequestUseCase(this.verificationRepository);

  Future<VerificationRequest> invoke(String sessionId) async {
    await Future.delayed(kDefaultMockDelay);
    return verificationRepository.getRequest(sessionId);
  }
}
