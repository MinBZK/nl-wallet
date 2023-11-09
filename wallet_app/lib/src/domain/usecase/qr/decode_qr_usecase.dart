import 'package:mobile_scanner/mobile_scanner.dart';

import '../../model/navigation/navigation_request.dart';

abstract class DecodeQrUseCase {
  Future<NavigationRequest?> invoke(Barcode barcode);
}
