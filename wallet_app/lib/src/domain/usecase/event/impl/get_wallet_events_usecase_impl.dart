import '../../../../data/repository/event/wallet_event_repository.dart';
import '../../../model/event/wallet_event.dart';
import '../get_wallet_events_usecase.dart';

class GetWalletEventsUseCaseImpl implements GetWalletEventsUseCase {
  final WalletEventRepository walletEventRepository;

  GetWalletEventsUseCaseImpl(this.walletEventRepository);

  /// Returns all wallet cards [WalletEvent]s, sorted by date DESC (newest first)
  @override
  Future<List<WalletEvent>> invoke() async => walletEventRepository.getEvents();
}
