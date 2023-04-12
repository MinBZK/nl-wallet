import '../../../../domain/model/sign_request.dart';
import '../sign_request_repository.dart';

class SignRequestRepositoryImpl implements SignRequestRepository {
  SignRequestRepositoryImpl();

  @override
  Future<SignRequest> getRequest(String sessionId) {
    // TODO: implement getRequest
    throw UnimplementedError();
  }
}
