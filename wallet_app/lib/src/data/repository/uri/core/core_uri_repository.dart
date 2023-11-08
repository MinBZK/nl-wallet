import '../../../../../bridge_generated.dart';
import '../../../../domain/model/navigation/navigation_request.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../uri_repository.dart';

class CoreUriRepository implements UriRepository {
  final TypedWalletCore _walletCore;

  CoreUriRepository(this._walletCore);

  @override
  Future<NavigationRequest> processUri(Uri uri) async {
    final uriType = await _walletCore.identifyUri(uri.toString());
    switch (uriType) {
      case IdentifyUriResult.PidIssuance:
        return PidIssuanceNavigationRequest(uri.toString());
      case IdentifyUriResult.Disclosure:
        return DisclosureNavigationRequest(uri.toString());
    }
  }
}
