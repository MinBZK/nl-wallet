import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class GetVersionStringUseCase extends WalletUseCase {
  Future<Result<String>> invoke();
}
