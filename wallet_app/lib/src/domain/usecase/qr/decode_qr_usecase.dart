import 'package:mobile_scanner/mobile_scanner.dart';

import '../../model/navigation/navigation_request.dart';
import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class DecodeQrUseCase extends WalletUseCase {
  Future<Result<NavigationRequest>> invoke(Barcode barcode);
}
