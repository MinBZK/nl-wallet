import '../../../../data/repository/event/wallet_event_repository.dart';
import '../../../model/event/wallet_event.dart';
import '../observe_recent_history_usecase.dart';

class ObserveRecentHistoryUseCaseImpl implements ObserveRecentHistoryUseCase {
  final WalletEventRepository walletEventRepository;

  ObserveRecentHistoryUseCaseImpl(this.walletEventRepository);

  @override
  Stream<List<WalletEvent>> invoke() => walletEventRepository.observeRecentEvents();
}
