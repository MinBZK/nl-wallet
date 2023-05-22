import '../../../../data/repository/sign/sign_request_repository.dart';
import '../../../model/sign_request.dart';
import '../get_sign_request_usecase.dart';

class GetSignRequestUseCaseImpl implements GetSignRequestUseCase {
  final SignRequestRepository signRequestRepository;

  GetSignRequestUseCaseImpl(this.signRequestRepository);

  @override
  Future<SignRequest> invoke(String sessionId) async {
    return signRequestRepository.getRequest(sessionId);
  }
}
