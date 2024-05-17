import '../../../../data/repository/event/wallet_event_repository.dart';
import '../../../model/event/wallet_event.dart';
import '../get_wallet_events_for_card_usecase.dart';

class GetWalletEventsForCardUseCaseImpl implements GetWalletEventsForCardUseCase {
  final WalletEventRepository walletEventRepository;

  GetWalletEventsForCardUseCaseImpl(this.walletEventRepository);

  /// Returns all wallet cards [WalletEvent]s, sorted by date DESC (newest first)
  @override
  Future<List<WalletEvent>> invoke(String docType) async => await walletEventRepository.getEventsForCard(docType);
}
