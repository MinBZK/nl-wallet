import 'dart:convert';

import 'package:fimber/fimber.dart';

import '../../../../feature/issuance/argument/issuance_screen_argument.dart';
import '../../../../navigation/wallet_routes.dart';
import '../../../../wallet_constants.dart';
import '../../../model/navigation/navigation_request.dart';
import '../../../model/qr/edi_qr_code.dart';
import '../decode_uri_usecase.dart';

/// Takes a [Uri] and attempts to provide a [NavigationRequest] that contains
/// the information to navigate the user to the related destination.
///
/// Sample to trigger 'Marketplace' verify mock scenario (QR code) from the terminal:
/// adb shell am start -a android.intent.action.VIEW -d "walletdebuginteraction://deeplink#%7B%22id%22%3A%20%22MARKETPLACE_LOGIN%22%2C%22type%22%3A%20%22verify%22%7D" nl.rijksoverheid.edi.wallet
///
/// Sample to trigger deepdive test scenario from the terminal:
/// adb shell am start -a android.intent.action.VIEW -d "walletdebuginteraction://deepdive#home" nl.rijksoverheid.edi.wallet
class MockDecodeUriUseCase implements DecodeUriUseCase {
  MockDecodeUriUseCase();

  @override
  Future<NavigationRequest> invoke(Uri uri) async {
    if (uri.host == kDeeplinkHost || uri.path.startsWith(kDeeplinkPath)) {
      return _decodeDeeplink(uri);
    } else if (uri.host == kDeepDiveHost || uri.path.startsWith(kDeepDivePath)) {
      return _decodeDeepDive(uri);
    }
    throw UnsupportedError('Unknown uri: $uri');
  }

  NavigationRequest _decodeDeeplink(Uri uri) {
    try {
      final json = jsonDecode(Uri.decodeComponent(uri.fragment));
      //FIXME: So far the only entry point has been the QR scanner.
      //FIXME: We are (ab)using the same json structure for now. As such re-using this class here.
      final code = EdiQrCode.fromJson(json);
      String destination;
      Object? argument;
      switch (code.type) {
        case EdiQrType.issuance:
          destination = WalletRoutes.issuanceRoute;
          argument = IssuanceScreenArgument(sessionId: code.id).toMap();
          break;
        case EdiQrType.disclosure:
          destination = WalletRoutes.disclosureRoute;
          argument = code.id;
          break;
        case EdiQrType.sign:
          destination = WalletRoutes.signRoute;
          argument = code.id;
          break;
      }
      return GenericNavigationRequest(
        destination,
        argument: argument,
        navigatePrerequisites: [NavigationPrerequisite.walletUnlocked, NavigationPrerequisite.pidInitialized],
      );
    } catch (ex, stack) {
      Fimber.e('Failed to parse deeplink uri: $uri', ex: ex, stacktrace: stack);
      throw UnsupportedError('Unknown uri: $uri');
    }
  }

  NavigationRequest _decodeDeepDive(Uri uri) {
    if (uri.hasFragment && uri.fragment == 'home') {
      return GenericNavigationRequest(
        WalletRoutes.homeRoute,
        argument: null,
        preNavigationActions: [PreNavigationAction.setupMockedWallet],
      );
    } else {
      Fimber.i('Unhandled deep dive uri: $uri');
      throw UnsupportedError('Unknown uri: $uri');
    }
  }
}
