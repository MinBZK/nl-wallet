import 'package:fimber/fimber.dart';
import 'package:mobile_scanner/mobile_scanner.dart';

import '../../../../data/repository/qr/qr_repository.dart';
import '../../../model/navigation/navigation_request.dart';
import '../decode_qr_usecase.dart';

class DecodeQrUseCaseImpl implements DecodeQrUseCase {
  final QrRepository _qrRepository;

  DecodeQrUseCaseImpl(this._qrRepository);

  @override
  Future<NavigationRequest?> invoke(Barcode barcode) async {
    try {
      return await _qrRepository.processBarcode(barcode);
    } catch (ex) {
      Fimber.e('Could not parse barcode: $barcode', ex: ex);
      return null;
    }
  }
}
