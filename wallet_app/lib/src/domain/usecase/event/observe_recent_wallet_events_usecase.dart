import '../../model/event/wallet_event.dart';

abstract class ObserveRecentWalletEventsUseCase {
  Stream<List<WalletEvent>> invoke();
}
