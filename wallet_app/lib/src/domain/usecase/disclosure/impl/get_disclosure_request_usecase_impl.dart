import '../../../../data/repository/disclosure/disclosure_request_repository.dart';
import '../get_disclosure_request_usecase.dart';

class GetDisclosureRequestUseCaseImpl extends GetDisclosureRequestUseCase {
  final DisclosureRequestRepository disclosureRequestRepository;

  GetDisclosureRequestUseCaseImpl(this.disclosureRequestRepository);

  @override
  Future<DisclosureRequest> invoke(String sessionId) async {
    return disclosureRequestRepository.getRequest(sessionId);
  }
}
