import 'package:wallet_core/core.dart';

import '../../../../../environment.dart';
import '../../../../domain/model/navigation/navigation_request.dart';
import '../../../../feature/disclosure/argument/disclosure_screen_argument.dart';
import '../../../../feature/issuance/argument/issuance_screen_argument.dart';
import '../../../../feature/sign/argument/sign_screen_argument.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../uri_repository.dart';

class CoreUriRepository implements UriRepository {
  final TypedWalletCore _walletCore;

  CoreUriRepository(this._walletCore);

  @override
  Future<NavigationRequest> processUri(Uri inputUri) async {
    final uri = inputUri.toString();
    if (Environment.mockRepositories) {
      // When wallet_core supports sign requests, this logic should be removed.
      if (uri.contains('sign')) return NavigationRequest.sign(argument: SignScreenArgument(uri: uri));
    }
    final uriType = await _walletCore.identifyUri(uri);
    switch (uriType) {
      case IdentifyUriResult.PidIssuance:
        return NavigationRequest.pidIssuance(uri);
      case IdentifyUriResult.PidRenewal:
        return NavigationRequest.pidRenewal(uri);
      case IdentifyUriResult.PinRecovery:
        return NavigationRequest.pinRecovery(uri);
      case IdentifyUriResult.Disclosure:
        return NavigationRequest.disclosure(argument: DisclosureScreenArgument(uri: uri, isQrCode: false));
      case IdentifyUriResult.DisclosureBasedIssuance:
        return NavigationRequest.issuance(argument: IssuanceScreenArgument(uri: uri, isQrCode: false));
      case IdentifyUriResult.Transfer:
        return NavigationRequest.walletTransferSource(uri);
    }
  }
}
