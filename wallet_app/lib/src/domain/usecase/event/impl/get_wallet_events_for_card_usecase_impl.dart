import '../../../../data/repository/event/wallet_event_repository.dart';
import '../../../model/event/wallet_event.dart';
import '../../../model/result/result.dart';
import '../get_wallet_events_for_card_usecase.dart';

class GetWalletEventsForCardUseCaseImpl extends GetWalletEventsForCardUseCase {
  final WalletEventRepository _walletEventRepository;

  GetWalletEventsForCardUseCaseImpl(this._walletEventRepository);

  /// Returns all wallet cards [WalletEvent]s, sorted by date DESC (newest first)
  @override
  Future<Result<List<WalletEvent>>> invoke(String attestationId) async {
    return tryCatch(
      () async => _walletEventRepository.getEventsForCard(attestationId),
      'Failed to resolve events for card: $attestationId',
    );
  }
}
