import '../../model/event/wallet_event.dart';

abstract class GetWalletEventsUseCase {
  Future<List<WalletEvent>> invoke();
}
