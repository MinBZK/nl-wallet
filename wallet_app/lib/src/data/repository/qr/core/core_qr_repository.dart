import 'package:wallet_core/core.dart';
import 'package:mobile_scanner/mobile_scanner.dart';

import '../../../../domain/model/navigation/navigation_request.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../qr_repository.dart';

class CoreQrRepository implements QrRepository {
  final TypedWalletCore _walletCore;

  CoreQrRepository(this._walletCore);

  @override
  Future<NavigationRequest> processBarcode(Barcode barcode) => _processRawValue(barcode.rawValue!);

  Future<NavigationRequest> _processRawValue(String rawValue) async {
    final uriType = await _walletCore.identifyUri(rawValue);
    switch (uriType) {
      case IdentifyUriResult.PidIssuance:
        return PidIssuanceNavigationRequest(rawValue);
      case IdentifyUriResult.Disclosure:
        return DisclosureNavigationRequest(rawValue);
    }
  }
}
