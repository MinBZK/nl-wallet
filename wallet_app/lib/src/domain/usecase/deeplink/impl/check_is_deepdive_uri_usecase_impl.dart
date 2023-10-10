import '../../../../wallet_constants.dart';
import '../check_is_deepdive_uri_usecase.dart';

class CheckIsDeepdiveUriUseCaseImpl implements CheckIsDeepdiveUriUseCase {
  CheckIsDeepdiveUriUseCaseImpl();

  @override
  bool invoke(Uri uri) {
    if (uri.host == kDeepDiveHost) return true;
    if (uri.path.startsWith(kDeepDivePath)) return true;
    return false;
  }
}
