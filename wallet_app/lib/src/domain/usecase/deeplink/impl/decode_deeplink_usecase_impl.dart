import 'dart:convert';

import 'package:fimber/fimber.dart';

import '../../../../feature/issuance/argument/issuance_screen_argument.dart';
import '../../../../wallet_routes.dart';
import '../../../model/navigation/navigation_request.dart';
import '../../../model/qr/edi_qr_code.dart';
import '../decode_deeplink_usecase.dart';

/// Takes a [Uri] and attempts to provide a [NavigationRequest] that contains
/// the information to navigate the user to the related destination.
///
/// Sample to trigger marketplace verify mock from the terminal:
/// adb shell am start -a android.intent.action.VIEW -d "walletdebuginteraction://deeplink#%7B%22id%22%3A%20%22MARKETPLACE_LOGIN%22%2C%22type%22%3A%20%22verify%22%7D" nl.rijksoverheid.edi.wallet
class DecodeDeeplinkUseCaseImpl implements DecodeDeeplinkUseCase {
  DecodeDeeplinkUseCaseImpl();

  @override
  NavigationRequest? invoke(Uri uri) {
    try {
      final json = jsonDecode(Uri.decodeComponent(uri.fragment));
      //FIXME: So far the only entry point has been the QR scanner.
      //FIXME: We are (ab)using the same json structure for now. As such re-using this class here.
      final code = EdiQrCode.fromJson(json);
      String destination;
      Object? argument;
      switch (code.type) {
        case EdiQrType.issue:
          destination = WalletRoutes.issuanceRoute;
          argument = IssuanceScreenArgument(sessionId: code.id).toMap();
          break;
        case EdiQrType.verify:
          destination = WalletRoutes.verificationRoute;
          argument = code.id;
          break;
        case EdiQrType.sign:
          destination = WalletRoutes.signRoute;
          argument = code.id;
          break;
      }
      return NavigationRequest(destination, argument: argument);
    } catch (ex, stack) {
      Fimber.e('Failed to parse uri: $uri', ex: ex, stacktrace: stack);
      return null;
    }
  }
}
