import '../../../feature/disclosure/model/disclosure_request.dart';

abstract class GetDisclosureRequestUseCase {
  Future<DisclosureRequest> invoke(String sessionId);
}
