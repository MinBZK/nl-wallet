import 'package:mobile_scanner/mobile_scanner.dart';

import '../../../../data/repository/qr/qr_repository.dart';
import '../../../model/navigation/navigation_request.dart';
import '../../../model/result/result.dart';
import '../decode_qr_usecase.dart';

class DecodeQrUseCaseImpl extends DecodeQrUseCase {
  final QrRepository _qrRepository;

  DecodeQrUseCaseImpl(this._qrRepository);

  @override
  Future<Result<NavigationRequest>> invoke(Barcode barcode) async {
    return tryCatch(
      () async => _qrRepository.processBarcode(barcode),
      'Could not decode barcode: $barcode',
    );
  }
}
