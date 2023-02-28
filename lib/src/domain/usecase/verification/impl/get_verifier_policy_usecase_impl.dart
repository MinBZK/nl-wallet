import '../../../../data/repository/verification/verification_request_repository.dart';
import '../../../model/policy/policy.dart';
import '../get_verifier_policy_usecase.dart';

class GetVerifierPolicyUseCaseImpl implements GetVerifierPolicyUseCase {
  final VerificationRequestRepository verificationRepository;

  GetVerifierPolicyUseCaseImpl(this.verificationRepository);

  @override
  Future<Policy> invoke(String sessionId) async {
    return (await verificationRepository.getRequest(sessionId)).interactionPolicy;
  }
}
