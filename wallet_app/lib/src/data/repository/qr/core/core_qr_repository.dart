import '../../../../domain/model/qr/qr_request.dart';
import '../qr_repository.dart';

class CoreQrRepository implements QrRepository {
  CoreQrRepository();

  @override
  Future<QrRequest> getRequest(String rawValue) {
    // TODO: implement getRequest
    throw UnimplementedError();
  }
}
