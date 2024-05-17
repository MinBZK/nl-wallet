import '../../model/event/wallet_event.dart';

abstract class ObserveRecentHistoryUseCase {
  Stream<List<WalletEvent>> invoke();
}
