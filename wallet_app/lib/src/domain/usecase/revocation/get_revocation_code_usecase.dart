import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class GetRevocationCodeUseCase extends WalletUseCase {
  Future<Result<String>> invoke(String pin);
}
