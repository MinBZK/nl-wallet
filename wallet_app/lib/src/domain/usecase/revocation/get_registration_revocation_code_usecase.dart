import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class GetRegistrationRevocationCodeUseCase extends WalletUseCase {
  Future<Result<String>> invoke();
}
