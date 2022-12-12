import '../../../data/repository/sign/sign_request_repository.dart';
import '../../../wallet_constants.dart';
import '../../model/sign_request.dart';

class GetSignRequestUseCase {
  final SignRequestRepository signRequestRepository;

  GetSignRequestUseCase(this.signRequestRepository);

  Future<SignRequest> invoke(String sessionId) async {
    await Future.delayed(kDefaultMockDelay);
    return signRequestRepository.getRequest(sessionId);
  }
}
