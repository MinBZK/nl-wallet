import 'package:mobile_scanner/mobile_scanner.dart';

import '../../../domain/model/navigation/navigation_request.dart';

abstract class QrRepository {
  Future<NavigationRequest> processBarcode(Barcode barcode);
}
