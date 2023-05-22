import '../../../../data/repository/verification/verification_request_repository.dart';
import '../../../../feature/verification/model/verification_request.dart';
import '../get_verification_request_usecase.dart';

class GetVerificationRequestUseCaseImpl extends GetVerificationRequestUseCase {
  final VerificationRequestRepository verificationRepository;

  GetVerificationRequestUseCaseImpl(this.verificationRepository);

  @override
  Future<VerificationRequest> invoke(String sessionId) async {
    return verificationRepository.getRequest(sessionId);
  }
}
