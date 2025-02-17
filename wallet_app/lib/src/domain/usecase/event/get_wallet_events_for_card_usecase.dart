import '../../model/event/wallet_event.dart';
import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class GetWalletEventsForCardUseCase extends WalletUseCase {
  Future<Result<List<WalletEvent>>> invoke(String docType);
}
