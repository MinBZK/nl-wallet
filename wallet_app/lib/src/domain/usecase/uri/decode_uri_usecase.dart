import '../../model/navigation/navigation_request.dart';
import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class DecodeUriUseCase extends WalletUseCase {
  Future<Result<NavigationRequest>> invoke(Uri uri);
}
