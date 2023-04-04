import '../../../domain/model/sign_request.dart';

abstract class SignRequestRepository {
  Future<SignRequest> getRequest(String sessionId);
}
