import '../../model/event/wallet_event.dart';
import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class GetWalletEventsForPidUseCase extends WalletUseCase {
  Future<Result<List<WalletEvent>>> invoke();
}
