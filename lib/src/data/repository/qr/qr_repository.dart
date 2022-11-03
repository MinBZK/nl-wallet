import '../../../domain/model/qr/qr_request.dart';

abstract class QrRepository {
  Future<QrRequest> getRequest(String rawValue);
}
