import '../../../data/repository/verification/verification_request_repository.dart';
import '../../model/policy/policy.dart';

class GetVerifierPolicyUseCase {
  final VerificationRequestRepository verificationRepository;

  GetVerifierPolicyUseCase(this.verificationRepository);

  Future<Policy> invoke(String sessionId) async {
    return (await verificationRepository.getRequest(sessionId)).interactionPolicy;
  }
}
