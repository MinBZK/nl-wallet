import '../../../../../bridge_generated.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../../../model/navigation/navigation_request.dart';
import '../decode_uri_usecase.dart';

class DecodeUriUseCaseImpl implements DecodeUriUseCase {
  final TypedWalletCore _walletCore;

  DecodeUriUseCaseImpl(this._walletCore);

  @override
  Future<NavigationRequest> invoke(Uri uri) async {
    final uriType = await _walletCore.identifyUri(uri.toString());
    switch (uriType) {
      case IdentifyUriResult.PidIssuance:
        return PidIssuanceNavigationRequest(uri);
      case IdentifyUriResult.Disclosure:
        return DisclosureNavigationRequest(uri);
    }
  }
}
