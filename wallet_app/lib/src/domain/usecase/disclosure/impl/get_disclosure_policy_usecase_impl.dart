import '../../../../data/repository/disclosure/disclosure_request_repository.dart';
import '../../../model/policy/policy.dart';
import '../get_disclosure_policy_usecase.dart';

class GetDisclosurePolicyUseCaseImpl implements GetDisclosurePolicyUseCase {
  final DisclosureRequestRepository disclosureRequestRepository;

  GetDisclosurePolicyUseCaseImpl(this.disclosureRequestRepository);

  @override
  Future<Policy> invoke(String sessionId) async {
    return (await disclosureRequestRepository.getRequest(sessionId)).interactionPolicy;
  }
}
