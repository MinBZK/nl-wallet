import '../../model/event/wallet_event.dart';
import '../wallet_usecase.dart';

abstract class GetMostRecentWalletEventUseCase extends WalletUseCase {
  Future<WalletEvent?> invoke();
}
