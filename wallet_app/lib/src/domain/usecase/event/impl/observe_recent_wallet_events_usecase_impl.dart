import '../../../../data/repository/event/wallet_event_repository.dart';
import '../../../model/event/wallet_event.dart';
import '../observe_recent_wallet_events_usecase.dart';

class ObserveRecentWalletEventsUseCaseImpl implements ObserveRecentWalletEventsUseCase {
  final WalletEventRepository walletEventRepository;

  ObserveRecentWalletEventsUseCaseImpl(this.walletEventRepository);

  @override
  Stream<List<WalletEvent>> invoke() => walletEventRepository.observeRecentEvents();
}
