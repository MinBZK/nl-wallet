import 'package:fimber/fimber.dart';

import '../../../../data/repository/event/wallet_event_repository.dart';
import '../../../model/event/wallet_event.dart';
import '../get_most_recent_wallet_event_usecase.dart';

class GetMostRecentWalletEventUsecaseImpl extends GetMostRecentWalletEventUseCase {
  final WalletEventRepository _walletEventRepository;

  GetMostRecentWalletEventUsecaseImpl(this._walletEventRepository);

  @override
  Future<WalletEvent?> invoke() async {
    try {
      final events = await _walletEventRepository.getEvents();
      return events.first;
    } catch (ex) {
      Fimber.e('Failed to get last event', ex: ex);
      return null;
    }
  }
}
