import 'dart:collection';

import '../../model/event/wallet_event.dart';
import '../../model/event/wallet_events_page.dart';
import '../../model/result/result.dart';
import '../wallet_usecase.dart';

/// Fetches [page] (of size [pageSize]) and merges it into [currentPages].
///
/// Filters out PID events that are logical duplicates of an event already present, both
/// within the fetched page and across the boundary with the last page in [currentPages].
abstract class GetWalletEventsPageUseCase extends WalletUseCase {
  Future<Result<WalletEventsPage>> invoke({
    required int page,
    required int pageSize,
    required SplayTreeMap<int, List<WalletEvent>> currentPages,
  });
}
