import '../../../../data/repository/event/wallet_event_repository.dart';
import '../../../model/event/wallet_event.dart';
import '../../wallet_usecase.dart';
import '../observe_recent_wallet_events_usecase.dart';

class ObserveRecentWalletEventsUseCaseImpl extends ObserveRecentWalletEventsUseCase {
  final WalletEventRepository _walletEventRepository;

  ObserveRecentWalletEventsUseCaseImpl(this._walletEventRepository);

  @override
  Stream<List<WalletEvent>> invoke() =>
      _walletEventRepository.observeRecentEvents().handleAppError('Error while observing wallet events');
}
