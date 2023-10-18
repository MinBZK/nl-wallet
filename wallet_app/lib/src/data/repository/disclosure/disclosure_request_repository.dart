import '../../../feature/disclosure/model/disclosure_request.dart';

export '../../../feature/disclosure/model/disclosure_request.dart';

abstract class DisclosureRequestRepository {
  Future<DisclosureRequest> getRequest(String sessionId);
}
