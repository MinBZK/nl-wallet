import '../../model/result/result.dart';
import '../wallet_usecase.dart';

export '../../model/result/result.dart';

/// Check the provided pin, optionally providing a returnUrl (e.g. when using pin to accept same device disclosure).
abstract class CheckPinUseCase extends WalletUseCase {
  Future<Result<String? /*returnUrl*/ >> invoke(String pin);
}
