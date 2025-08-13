import 'dart:convert';

import 'package:fimber/fimber.dart';
import 'package:meta/meta.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:wallet_core/core.dart';

import '../../../../../environment.dart';
import '../../../../domain/model/navigation/navigation_request.dart';
import '../../../../domain/model/qr/edi_qr_code.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../qr_repository.dart';

class CoreQrRepository implements QrRepository {
  final TypedWalletCore _walletCore;

  CoreQrRepository(this._walletCore);

  @override
  Future<NavigationRequest> processBarcode(Barcode barcode) {
    final legacyValue = legacyQrToDeeplink(barcode);
    return _processRawValue(legacyValue ?? barcode.rawValue!);
  }

  /// Attempt to convert a legacy style json encoded scenario to a deeplink url that we can process normally.
  /// Sample input: {"id":"DRIVING_LICENSE","type":"issue"}
  /// Returns null if the conversion failed or was intentionally not attempted (on non-mock builds).
  @visibleForTesting
  String? legacyQrToDeeplink(Barcode barcode, {bool forceConversion = false}) {
    if (!Environment.mockRepositories && !forceConversion) return null;
    try {
      EdiQrCode.fromJson(jsonDecode(barcode.rawValue!));
      // No exception, so create the deeplink uri that we can process normally
      final String url = 'walletdebuginteraction://deeplink#${barcode.rawValue}';
      return Uri.parse(url).toString(); // uri encode the content
    } catch (ex) {
      Fimber.e('Failed to extract process as EdiQrCode. Contents: ${barcode.rawValue}');
    }
    return null;
  }

  Future<NavigationRequest> _processRawValue(String rawValue) async {
    if (Environment.mockRepositories) {
      // When wallet_core supports issue/sign requests, this logic should be removed.
      if (rawValue.contains('issue')) return IssuanceNavigationRequest(rawValue);
      if (rawValue.contains('sign')) return SignNavigationRequest(rawValue);
    }
    final uriType = await _walletCore.identifyUri(rawValue);
    switch (uriType) {
      case IdentifyUriResult.PidIssuance:
        return PidIssuanceNavigationRequest(rawValue);
      case IdentifyUriResult.PidRenewal:
        return PidRenewalNavigationRequest(rawValue);
      case IdentifyUriResult.PinRecovery:
        throw UnimplementedError('PinRecovery not yet supported');
      case IdentifyUriResult.Disclosure:
        return DisclosureNavigationRequest(rawValue, isQrCode: true);
      case IdentifyUriResult.DisclosureBasedIssuance:
        return IssuanceNavigationRequest(rawValue, isQrCode: true);
    }
  }
}
