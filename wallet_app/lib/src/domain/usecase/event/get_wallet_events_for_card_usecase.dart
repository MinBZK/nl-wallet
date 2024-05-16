import '../../model/event/wallet_event.dart';

abstract class GetWalletEventsForCardUseCase {
  Future<List<WalletEvent>> invoke(String docType);
}
