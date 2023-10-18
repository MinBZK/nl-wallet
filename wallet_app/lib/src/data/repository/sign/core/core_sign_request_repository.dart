import '../../../../domain/model/sign_request.dart';
import '../sign_request_repository.dart';

class CoreSignRequestRepository implements SignRequestRepository {
  CoreSignRequestRepository();

  @override
  Future<SignRequest> getRequest(String sessionId) {
    // TODO: implement getRequest
    throw UnimplementedError();
  }
}
