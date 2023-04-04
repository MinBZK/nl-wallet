import 'package:fimber/fimber.dart';
import 'package:mobile_scanner/mobile_scanner.dart';

import '../../../../data/repository/qr/qr_repository.dart';
import '../../../model/qr/qr_request.dart';
import '../decode_qr_usecase.dart';

class DecodeQrUseCaseImpl implements DecodeQrUseCase {
  final QrRepository qrRepository;

  DecodeQrUseCaseImpl(this.qrRepository);

  @override
  Future<QrRequest?> invoke(Barcode barcode) async {
    try {
      return await qrRepository.getRequest(barcode.rawValue!);
    } catch (ex, stack) {
      Fimber.e('Failed to parse barcode: $barcode', ex: ex, stacktrace: stack);
      return null;
    }
  }
}
