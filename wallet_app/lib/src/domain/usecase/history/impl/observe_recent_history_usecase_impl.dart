import '../../../../data/repository/event/wallet_event_repository.dart';
import '../../../model/event/wallet_event.dart';
import '../../wallet_usecase.dart';
import '../observe_recent_history_usecase.dart';

class ObserveRecentHistoryUseCaseImpl extends ObserveRecentHistoryUseCase {
  final WalletEventRepository _walletEventRepository;

  ObserveRecentHistoryUseCaseImpl(this._walletEventRepository);

  @override
  Stream<List<WalletEvent>> invoke() =>
      _walletEventRepository.observeRecentEvents().handleAppError('Error while observing card details');
}
