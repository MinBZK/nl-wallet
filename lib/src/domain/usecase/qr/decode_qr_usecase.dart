import 'package:mobile_scanner/mobile_scanner.dart';

import '../../model/qr/qr_request.dart';

abstract class DecodeQrUseCase {
  Future<QrRequest?> invoke(Barcode barcode);
}
