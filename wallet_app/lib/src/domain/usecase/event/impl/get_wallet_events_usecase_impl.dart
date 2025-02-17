import '../../../../data/repository/event/wallet_event_repository.dart';
import '../../../model/event/wallet_event.dart';
import '../../../model/result/result.dart';
import '../get_wallet_events_usecase.dart';

class GetWalletEventsUseCaseImpl extends GetWalletEventsUseCase {
  final WalletEventRepository _walletEventRepository;

  GetWalletEventsUseCaseImpl(this._walletEventRepository);

  /// Returns all wallet cards [WalletEvent]s, sorted by date DESC (newest first)
  @override
  Future<Result<List<WalletEvent>>> invoke() async {
    return tryCatch(
      () async => _walletEventRepository.getEvents(),
      'Failed to get wallet events',
    );
  }
}
