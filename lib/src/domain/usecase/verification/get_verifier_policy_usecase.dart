import '../../../data/repository/verification/verification_request_repository.dart';
import '../../model/policy/interaction_policy.dart';

class GetVerifierPolicyUseCase {
  final VerificationRequestRepository verificationRepository;

  GetVerifierPolicyUseCase(this.verificationRepository);

  Future<InteractionPolicy> invoke(String sessionId) async {
    return (await verificationRepository.getRequest(sessionId)).interactionPolicy;
  }
}
