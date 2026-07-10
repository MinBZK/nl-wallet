import '../../../../../environment.dart';
import '../../../../domain/model/navigation/navigation_request.dart';
import '../../../../feature/sign/argument/sign_screen_argument.dart';
import '../../../../navigation/navigation_request_resolver.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../uri_repository.dart';

class CoreUriRepository implements UriRepository {
  final TypedWalletCore _walletCore;

  CoreUriRepository(this._walletCore);

  @override
  Future<NavigationRequest> processUri(Uri inputUri) async {
    final uri = inputUri.toString();
    if (Environment.mockRepositories) {
      // Once wallet_core supports sign requests, this logic should be removed.
      if (uri.contains('sign')) return NavigationRequest.sign(argument: SignScreenArgument(uri: uri));
    }
    final uriType = await _walletCore.identifyUri(uri);
    return resolveNavigationRequest(uriType, uri, isQrCode: false);
  }
}
