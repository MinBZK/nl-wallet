import '../../model/event/wallet_event.dart';
import '../wallet_usecase.dart';

abstract class ObserveRecentWalletEventsUseCase extends WalletUseCase {
  Stream<List<WalletEvent>> invoke();
}
