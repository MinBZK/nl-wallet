import '../../../data/repository/verification/verification_request_repository.dart';
import '../../../feature/verification/model/verifier_policy.dart';

class GetVerifierPolicyUseCase {
  final VerificationRequestRepository verificationRepository;

  GetVerifierPolicyUseCase(this.verificationRepository);

  Future<VerifierPolicy> invoke(String sessionId) async {
    return (await verificationRepository.getRequest(sessionId)).policy;
  }
}
